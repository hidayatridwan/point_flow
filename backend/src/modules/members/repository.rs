use serde_json::json;
use sqlx::PgPool;

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::{
        CreateMemberRequest, MemberQuery, MemberResponse, ResetMemberPasswordRequest,
        UpdateMemberRequest,
    },
    security,
    services::{audit, validate_required},
};

const MEMBER_SELECT: &str = r#"
    SELECT
        id,
        member_no,
        full_name,
        phone,
        email,
        address,
        joined_at,
        is_active,
        notes,
        created_at,
        updated_at
    FROM members
"#;

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    })
}

fn map_unique_violation(error: sqlx::Error) -> AppError {
    match &error {
        sqlx::Error::Database(db_error) if db_error.code().as_deref() == Some("23505") => {
            AppError::Conflict("phone number is already registered".to_string())
        }
        _ => AppError::Database(error),
    }
}

fn normalize_email(email: Option<String>) -> AppResult<Option<String>> {
    match normalize_optional_string(email) {
        Some(email) => {
            if !email.contains('@') {
                return Err(AppError::Validation(
                    "email must be a valid email".to_string(),
                ));
            }
            Ok(Some(email))
        }
        None => Ok(None),
    }
}

pub async fn list_members(pool: &PgPool, query: MemberQuery) -> AppResult<Vec<MemberResponse>> {
    let search = query.search_pattern();
    let sql = format!(
        "{MEMBER_SELECT}
        WHERE ($1 = '%%'
            OR lower(full_name) LIKE $1
            OR lower(phone) LIKE $1
            OR lower(member_no) LIKE $1)
            AND ($2::boolean IS NULL OR is_active = $2)
        ORDER BY full_name, id
        LIMIT $3 OFFSET $4"
    );

    sqlx::query_as::<_, MemberResponse>(&sql)
        .bind(search)
        .bind(query.is_active)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_member(pool: &PgPool, id: i64) -> AppResult<MemberResponse> {
    let sql = format!("{MEMBER_SELECT} WHERE id = $1");

    sqlx::query_as::<_, MemberResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_member(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateMemberRequest,
) -> AppResult<MemberResponse> {
    validate_required(&request.full_name, "full_name")?;
    validate_required(&request.phone, "phone")?;
    let email = normalize_email(request.email)?;

    let full_name = request.full_name.trim().to_string();
    let phone = request.phone.trim().to_string();
    let duplicate: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM members WHERE phone = $1)")
            .bind(&phone)
            .fetch_one(pool)
            .await?;
    if duplicate {
        return Err(AppError::Conflict(
            "phone number is already registered".to_string(),
        ));
    }

    let password_hash = security::hash_password(&request.password)?;
    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO members (full_name, phone, email, password_hash, address, is_active, notes)
        VALUES ($1, $2, $3, $4, $5, COALESCE($6, TRUE), $7)
        RETURNING id
        "#,
    )
    .bind(&full_name)
    .bind(&phone)
    .bind(email)
    .bind(password_hash)
    .bind(normalize_optional_string(request.address))
    .bind(request.is_active)
    .bind(normalize_optional_string(request.notes))
    .fetch_one(pool)
    .await
    .map_err(map_unique_violation)?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(Some(user.id), "member.create", "member", Some(id))
            .with_metadata(json!({ "full_name": full_name, "phone": phone })),
    )
    .await;

    find_member(pool, id).await
}

pub async fn update_member(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: UpdateMemberRequest,
) -> AppResult<MemberResponse> {
    find_member(pool, id).await?;

    if let Some(full_name) = &request.full_name {
        validate_required(full_name, "full_name")?;
    }
    let email = normalize_email(request.email)?;
    if let Some(phone) = &request.phone {
        validate_required(phone, "phone")?;
        let phone = phone.trim().to_string();
        let duplicate: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM members WHERE phone = $1 AND id <> $2)",
        )
        .bind(&phone)
        .bind(id)
        .fetch_one(pool)
        .await?;
        if duplicate {
            return Err(AppError::Conflict(
                "phone number is already registered".to_string(),
            ));
        }
    }

    let updated_id: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE members
        SET
            full_name = COALESCE($2, full_name),
            phone = COALESCE($3, phone),
            email = COALESCE($4, email),
            address = COALESCE($5, address),
            is_active = COALESCE($6, is_active),
            notes = COALESCE($7, notes)
        WHERE id = $1
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(request.full_name.map(|value| value.trim().to_string()))
    .bind(request.phone.map(|value| value.trim().to_string()))
    .bind(email)
    .bind(normalize_optional_string(request.address))
    .bind(request.is_active)
    .bind(normalize_optional_string(request.notes))
    .fetch_optional(pool)
    .await
    .map_err(map_unique_violation)?;

    match updated_id {
        Some(id) => {
            audit::record_audit_log_best_effort(
                pool,
                audit::AuditLogRecord::success(Some(user.id), "member.update", "member", Some(id)),
            )
            .await;
            find_member(pool, id).await
        }
        None => Err(AppError::NotFound),
    }
}

pub async fn reset_member_password(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: ResetMemberPasswordRequest,
) -> AppResult<MemberResponse> {
    find_member(pool, id).await?;

    let password_hash = security::hash_password(&request.new_password)?;
    let updated_id: Option<i64> =
        sqlx::query_scalar("UPDATE members SET password_hash = $2 WHERE id = $1 RETURNING id")
            .bind(id)
            .bind(password_hash)
            .fetch_optional(pool)
            .await?;

    match updated_id {
        Some(id) => {
            audit::record_audit_log_best_effort(
                pool,
                audit::AuditLogRecord::success(
                    Some(user.id),
                    "member.reset_password",
                    "member",
                    Some(id),
                ),
            )
            .await;
            find_member(pool, id).await
        }
        None => Err(AppError::NotFound),
    }
}

/// Hapus cerdas: nonaktifkan bila member sudah dipakai transaksi,
/// hapus permanen bila belum dipakai.
pub async fn delete_member(pool: &PgPool, user: &AuthUser, id: i64) -> AppResult<MemberResponse> {
    let member = find_member(pool, id).await?;

    let used: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sales WHERE member_id = $1)
            OR EXISTS(SELECT 1 FROM online_orders WHERE member_id = $1)",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    if used {
        let updated_id: Option<i64> =
            sqlx::query_scalar("UPDATE members SET is_active = FALSE WHERE id = $1 RETURNING id")
                .bind(id)
                .fetch_optional(pool)
                .await?;

        match updated_id {
            Some(id) => {
                audit::record_audit_log_best_effort(
                    pool,
                    audit::AuditLogRecord::success(
                        Some(user.id),
                        "member.deactivate",
                        "member",
                        Some(id),
                    ),
                )
                .await;
                find_member(pool, id).await
            }
            None => Err(AppError::NotFound),
        }
    } else {
        sqlx::query("DELETE FROM members WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        audit::record_audit_log_best_effort(
            pool,
            audit::AuditLogRecord::success(Some(user.id), "member.delete", "member", Some(id)),
        )
        .await;

        Ok(member)
    }
}
