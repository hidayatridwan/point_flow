use chrono::{Duration, Utc};
use sqlx::{PgPool, Row};

use crate::{
    error::{AppError, AppResult},
    models::{ChangePasswordRequest, MemberLoginRequest, MemberLoginResponse, MemberResponse},
    security,
};

use super::validate_required;

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

#[derive(Debug)]
pub struct MemberSession {
    pub response: MemberLoginResponse,
    pub refresh_token: String,
}

pub async fn login(
    pool: &PgPool,
    request: MemberLoginRequest,
    jwt_secret: &str,
    access_token_ttl_seconds: i64,
    refresh_token_ttl_seconds: i64,
) -> AppResult<MemberSession> {
    validate_required(&request.phone, "phone")?;
    validate_required(&request.password, "password")?;
    let login_phone = request.phone.trim().to_string();

    let row = sqlx::query("SELECT id, password_hash, is_active FROM members WHERE phone = $1")
        .bind(&login_phone)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let member_id: i64 = row.get("id");
    let password_hash: String = row.get("password_hash");
    let is_active: bool = row.get("is_active");

    // Telepon tak dikenal, password salah, dan member nonaktif semuanya 401
    // agar status akun tidak bisa di-enumerasi dari luar.
    if !is_active || !security::verify_password(&request.password, &password_hash)? {
        return Err(AppError::Unauthorized);
    }

    let member = find_active_member(pool, member_id).await?;
    create_session(
        pool,
        member,
        jwt_secret,
        access_token_ttl_seconds,
        refresh_token_ttl_seconds,
    )
    .await
}

pub async fn refresh(
    pool: &PgPool,
    refresh_token: &str,
    jwt_secret: &str,
    access_token_ttl_seconds: i64,
    refresh_token_ttl_seconds: i64,
) -> AppResult<MemberSession> {
    let current_token_hash = security::hash_refresh_token(refresh_token);
    let new_refresh_token = security::generate_refresh_token();
    let new_token_hash = security::hash_refresh_token(&new_refresh_token);
    let new_expires_at = Utc::now() + Duration::seconds(refresh_token_ttl_seconds);
    let mut tx = pool.begin().await?;

    let row = sqlx::query(
        r#"
        SELECT rt.id, rt.member_id
        FROM member_refresh_tokens rt
        JOIN members m ON m.id = rt.member_id
        WHERE rt.token_hash = $1
            AND rt.revoked_at IS NULL
            AND rt.expires_at > now()
            AND m.is_active = TRUE
        FOR UPDATE
        "#,
    )
    .bind(&current_token_hash)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let current_token_id: i64 = row.get("id");
    let member_id: i64 = row.get("member_id");

    let _new_token_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO member_refresh_tokens (member_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(member_id)
    .bind(&new_token_hash)
    .bind(new_expires_at)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("UPDATE member_refresh_tokens SET revoked_at = now() WHERE id = $1")
        .bind(current_token_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    let member = find_active_member(pool, member_id).await?;
    let access_token = security::create_token(
        member.id,
        security::MEMBER_TOKEN_ROLE,
        jwt_secret,
        access_token_ttl_seconds,
    )?;

    Ok(MemberSession {
        response: MemberLoginResponse {
            access_token,
            token_type: "Bearer",
            expires_in: access_token_ttl_seconds,
            member,
        },
        refresh_token: new_refresh_token,
    })
}

pub async fn logout(pool: &PgPool, refresh_token: &str) -> AppResult<()> {
    let token_hash = security::hash_refresh_token(refresh_token);
    sqlx::query(
        r#"
        UPDATE member_refresh_tokens
        SET revoked_at = now()
        WHERE token_hash = $1
            AND revoked_at IS NULL
        "#,
    )
    .bind(token_hash)
    .execute(pool)
    .await?;

    Ok(())
}

/// Ganti password member yang login. Semua refresh token dicabut sehingga
/// sesi lain ikut keluar (sama seperti ganti password staff).
pub async fn change_password(
    pool: &PgPool,
    member_id: i64,
    request: ChangePasswordRequest,
) -> AppResult<()> {
    validate_required(&request.current_password, "current_password")?;
    validate_required(&request.new_password, "new_password")?;

    let mut tx = pool.begin().await?;
    let password_hash = sqlx::query_scalar::<_, String>(
        "SELECT password_hash FROM members WHERE id = $1 AND is_active = TRUE FOR UPDATE",
    )
    .bind(member_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if !security::verify_password(&request.current_password, &password_hash)? {
        return Err(AppError::Unauthorized);
    }

    let new_password_hash = security::hash_password(&request.new_password)?;

    sqlx::query("UPDATE members SET password_hash = $2 WHERE id = $1")
        .bind(member_id)
        .bind(new_password_hash)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "UPDATE member_refresh_tokens SET revoked_at = now() WHERE member_id = $1 AND revoked_at IS NULL",
    )
    .bind(member_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

/// Profil member aktif untuk `/members/auth/me` dan prefill checkout.
pub async fn find_active_member(pool: &PgPool, id: i64) -> AppResult<MemberResponse> {
    let sql = format!("{MEMBER_SELECT} WHERE id = $1 AND is_active = TRUE");

    sqlx::query_as::<_, MemberResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::Unauthorized)
}

async fn create_session(
    pool: &PgPool,
    member: MemberResponse,
    jwt_secret: &str,
    access_token_ttl_seconds: i64,
    refresh_token_ttl_seconds: i64,
) -> AppResult<MemberSession> {
    let access_token = security::create_token(
        member.id,
        security::MEMBER_TOKEN_ROLE,
        jwt_secret,
        access_token_ttl_seconds,
    )?;
    let refresh_token = security::generate_refresh_token();
    let token_hash = security::hash_refresh_token(&refresh_token);
    let expires_at = Utc::now() + Duration::seconds(refresh_token_ttl_seconds);

    sqlx::query(
        r#"
        INSERT INTO member_refresh_tokens (member_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(member.id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(MemberSession {
        response: MemberLoginResponse {
            access_token,
            token_type: "Bearer",
            expires_in: access_token_ttl_seconds,
            member,
        },
        refresh_token,
    })
}
