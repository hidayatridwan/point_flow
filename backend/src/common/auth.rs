use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use sqlx::Row;

use crate::{
    app::AppState,
    error::{AppError, AppResult},
    security,
};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub role: String,
}

impl AuthUser {
    pub fn require_admin(&self) -> AppResult<()> {
        if self.role == "admin" {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }

    pub async fn require_permission(
        &self,
        pool: &sqlx::PgPool,
        permission_code: &str,
    ) -> AppResult<()> {
        self.require_any_permission(pool, &[permission_code]).await
    }

    pub async fn require_any_permission(
        &self,
        pool: &sqlx::PgPool,
        permission_codes: &[&str],
    ) -> AppResult<()> {
        if crate::services::rbac::user_has_any_permission(pool, self.id, permission_codes).await? {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        let claims = security::verify_token(token, &state.jwt_secret)?;
        // Token member-scope tidak berlaku di dashboard staff, sekalipun
        // numeric id-nya kebetulan sama dengan id user staff.
        if security::is_member_scope(&claims.role) {
            return Err(AppError::Unauthorized);
        }
        let row = sqlx::query(
            r#"
            SELECT r.code AS role
            FROM users u
            JOIN roles r ON r.id = u.role_id
            WHERE u.id = $1
                AND u.deleted_at IS NULL
                AND u.is_active = TRUE
                AND r.deleted_at IS NULL
            "#,
        )
        .bind(claims.sub)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

        Ok(Self {
            id: claims.sub,
            role: row.get("role"),
        })
    }
}

/// Identitas member yang login di storefront. Hanya menerima JWT
/// member-scope (`role == "member"`); token staff selalu ditolak di sini.
#[derive(Debug, Clone)]
pub struct AuthMember {
    pub id: i64,
}

impl FromRequestParts<AppState> for AuthMember {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        let claims = security::verify_token(token, &state.jwt_secret)?;
        if !security::is_member_scope(&claims.role) {
            return Err(AppError::Unauthorized);
        }
        let is_active: Option<bool> =
            sqlx::query_scalar("SELECT is_active FROM members WHERE id = $1")
                .bind(claims.sub)
                .fetch_optional(&state.db)
                .await?;
        if is_active != Some(true) {
            return Err(AppError::Unauthorized);
        }

        Ok(Self { id: claims.sub })
    }
}

/// `Authorization` opsional untuk checkout publik: tanpa header berarti guest
/// (`None`); bila header ada, token harus member valid atau 401 — token
/// kedaluwarsa/nonaktif/scope salah tidak pernah diturunkan diam-diam ke guest
/// agar atribusi SHU tidak hilang tanpa disadari.
#[derive(Debug, Clone)]
pub struct OptionalMember(pub Option<AuthMember>);

impl FromRequestParts<AppState> for OptionalMember {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let has_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| !value.trim().is_empty());
        if !has_header {
            return Ok(Self(None));
        }
        Ok(Self(Some(
            AuthMember::from_request_parts(parts, state).await?,
        )))
    }
}
