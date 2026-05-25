use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::{PgPool, Row};

use crate::{
    error::{AppError, AppResult},
    models::{ChangePasswordRequest, CreateUserRequest, LoginRequest, LoginResponse, UserResponse},
    security,
};

use super::{audit, rbac, validate_required, validate_status};

const USER_SELECT: &str = r#"
    SELECT
        u.id,
        u.name,
        u.email,
        r.id AS role_id,
        r.code AS role_code,
        r.name AS role_name,
        r.code AS role,
        ARRAY(
            SELECT p.code
            FROM role_permissions rp
            JOIN permissions p ON p.id = rp.permission_id
            WHERE rp.role_id = r.id
            ORDER BY p.code
        ) AS permissions,
        u.is_active,
        u.last_login_at,
        u.created_at,
        u.updated_at,
        u.deleted_at
    FROM users u
    JOIN roles r ON r.id = u.role_id
"#;

#[derive(Debug)]
pub struct AuthSession {
    pub response: LoginResponse,
    pub refresh_token: String,
}

pub async fn login(
    pool: &PgPool,
    request: LoginRequest,
    jwt_secret: &str,
    access_token_ttl_seconds: i64,
    refresh_token_ttl_seconds: i64,
) -> AppResult<AuthSession> {
    validate_required(&request.email, "email")?;
    validate_required(&request.password, "password")?;
    let login_email = request.email.trim().to_string();

    let row = sqlx::query(
        r#"
        SELECT u.id, u.password_hash, u.is_active
        FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE lower(u.email) = lower($1)
            AND u.deleted_at IS NULL
            AND r.deleted_at IS NULL
        "#,
    )
    .bind(&request.email)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let user_id: i64 = row.get("id");
    let password_hash: String = row.get("password_hash");
    let is_active: bool = row.get("is_active");

    if !is_active || !security::verify_password(&request.password, &password_hash)? {
        return Err(AppError::Unauthorized);
    }

    sqlx::query("UPDATE users SET last_login_at = now() WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    let user = find_user(pool, user_id).await?;
    let session = create_session(
        pool,
        user,
        jwt_secret,
        access_token_ttl_seconds,
        refresh_token_ttl_seconds,
    )
    .await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(session.response.user.id),
            "auth.login",
            "user",
            Some(session.response.user.id),
        )
        .with_resource_label(session.response.user.email.clone())
        .with_metadata(json!({ "email": login_email })),
    )
    .await;

    Ok(session)
}

pub async fn bootstrap_admin(
    pool: &PgPool,
    request: CreateUserRequest,
    jwt_secret: &str,
    access_token_ttl_seconds: i64,
    refresh_token_ttl_seconds: i64,
) -> AppResult<AuthSession> {
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    if user_count > 0 {
        return Err(AppError::Conflict(
            "bootstrap is only available before the first user is created".to_string(),
        ));
    }

    validate_required(&request.name, "name")?;
    validate_required(&request.email, "email")?;
    let role = request.role.clone().unwrap_or_else(|| "admin".to_string());
    validate_status(&role, &["admin"], "role")?;
    let (role_id, role_code) =
        rbac::resolve_role(pool, request.role_id, request.role, "admin").await?;
    if role_code != "admin" {
        return Err(AppError::Validation("role must be admin".to_string()));
    }
    let password_hash = security::hash_password(&request.password)?;

    let user_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO users (name, email, password_hash, role_id, role, is_active)
        VALUES ($1, $2, $3, $4, 'admin', TRUE)
        RETURNING id
        "#,
    )
    .bind(request.name.trim())
    .bind(request.email.trim())
    .bind(password_hash)
    .bind(role_id)
    .fetch_one(pool)
    .await?;

    let user = find_user(pool, user_id).await?;
    let session = create_session(
        pool,
        user,
        jwt_secret,
        access_token_ttl_seconds,
        refresh_token_ttl_seconds,
    )
    .await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(session.response.user.id),
            "auth.bootstrap_admin",
            "user",
            Some(session.response.user.id),
        )
        .with_resource_label(session.response.user.email.clone())
        .with_after_data(json!({
            "id": session.response.user.id,
            "email": session.response.user.email.clone(),
            "role": session.response.user.role.clone(),
            "is_active": session.response.user.is_active
        })),
    )
    .await;

    Ok(session)
}

pub async fn refresh(
    pool: &PgPool,
    refresh_token: &str,
    jwt_secret: &str,
    access_token_ttl_seconds: i64,
    refresh_token_ttl_seconds: i64,
) -> AppResult<AuthSession> {
    let current_token_hash = security::hash_refresh_token(refresh_token);
    let new_refresh_token = security::generate_refresh_token();
    let new_token_hash = security::hash_refresh_token(&new_refresh_token);
    let new_expires_at = Utc::now() + Duration::seconds(refresh_token_ttl_seconds);
    let mut tx = pool.begin().await?;

    let row = sqlx::query(
        r#"
        SELECT rt.id, rt.user_id, u.is_active
        FROM refresh_tokens rt
        JOIN users u ON u.id = rt.user_id
        JOIN roles r ON r.id = u.role_id
        WHERE rt.token_hash = $1
            AND rt.revoked_at IS NULL
            AND rt.expires_at > now()
            AND u.deleted_at IS NULL
            AND r.deleted_at IS NULL
        FOR UPDATE
        "#,
    )
    .bind(&current_token_hash)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let current_token_id: i64 = row.get("id");
    let user_id: i64 = row.get("user_id");
    let is_active: bool = row.get("is_active");
    if !is_active {
        return Err(AppError::Unauthorized);
    }

    let new_token_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(&new_token_hash)
    .bind(new_expires_at)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = now(), replaced_by_token_id = $2
        WHERE id = $1
        "#,
    )
    .bind(current_token_id)
    .bind(new_token_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let user = find_user(pool, user_id).await?;
    let access_token =
        security::create_token(user.id, &user.role, jwt_secret, access_token_ttl_seconds)?;

    Ok(AuthSession {
        response: LoginResponse {
            access_token,
            token_type: "Bearer",
            expires_in: access_token_ttl_seconds,
            user,
        },
        refresh_token: new_refresh_token,
    })
}

pub async fn logout(pool: &PgPool, refresh_token: &str) -> AppResult<()> {
    let token_hash = security::hash_refresh_token(refresh_token);
    let user_id = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT user_id
        FROM refresh_tokens
        WHERE token_hash = $1
            AND revoked_at IS NULL
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = now()
        WHERE token_hash = $1
            AND revoked_at IS NULL
        "#,
    )
    .bind(token_hash)
    .execute(pool)
    .await?;

    if let Some(user_id) = user_id {
        audit::record_audit_log_best_effort(
            pool,
            audit::AuditLogRecord::success(Some(user_id), "auth.logout", "user", Some(user_id)),
        )
        .await;
    }

    Ok(())
}

pub async fn change_password(
    pool: &PgPool,
    user_id: i64,
    request: ChangePasswordRequest,
) -> AppResult<()> {
    validate_required(&request.current_password, "current_password")?;
    validate_required(&request.new_password, "new_password")?;

    let mut tx = pool.begin().await?;
    let password_hash = sqlx::query_scalar::<_, String>(
        r#"
        SELECT password_hash
        FROM users
        WHERE id = $1
            AND deleted_at IS NULL
            AND is_active = TRUE
        FOR UPDATE
        "#,
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if !security::verify_password(&request.current_password, &password_hash)? {
        return Err(AppError::Unauthorized);
    }

    let new_password_hash = security::hash_password(&request.new_password)?;

    sqlx::query(
        r#"
        UPDATE users
        SET password_hash = $2
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(new_password_hash)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = now()
        WHERE user_id = $1
            AND revoked_at IS NULL
        "#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(user_id),
            "auth.change_password",
            "user",
            Some(user_id),
        ),
    )
    .await;

    Ok(())
}

pub async fn find_user(pool: &PgPool, id: i64) -> AppResult<UserResponse> {
    let query =
        format!("{USER_SELECT} WHERE u.id = $1 AND u.deleted_at IS NULL AND r.deleted_at IS NULL");

    sqlx::query_as::<_, UserResponse>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

async fn create_session(
    pool: &PgPool,
    user: UserResponse,
    jwt_secret: &str,
    access_token_ttl_seconds: i64,
    refresh_token_ttl_seconds: i64,
) -> AppResult<AuthSession> {
    let access_token =
        security::create_token(user.id, &user.role, jwt_secret, access_token_ttl_seconds)?;
    let refresh_token = security::generate_refresh_token();
    let token_hash = security::hash_refresh_token(&refresh_token);
    let expires_at = Utc::now() + Duration::seconds(refresh_token_ttl_seconds);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(user.id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(AuthSession {
        response: LoginResponse {
            access_token,
            token_type: "Bearer",
            expires_in: access_token_ttl_seconds,
            user,
        },
        refresh_token,
    })
}
