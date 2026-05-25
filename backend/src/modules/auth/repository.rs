use sqlx::PgPool;

use crate::{
    error::AppResult,
    models::{ChangePasswordRequest, CreateUserRequest, LoginRequest, UserResponse},
    services::auth as auth_service,
};

pub async fn login(
    pool: &PgPool,
    request: LoginRequest,
    jwt_secret: &str,
    access_token_ttl_seconds: i64,
    refresh_token_ttl_seconds: i64,
) -> AppResult<auth_service::AuthSession> {
    auth_service::login(
        pool,
        request,
        jwt_secret,
        access_token_ttl_seconds,
        refresh_token_ttl_seconds,
    )
    .await
}

pub async fn bootstrap_admin(
    pool: &PgPool,
    request: CreateUserRequest,
    jwt_secret: &str,
    access_token_ttl_seconds: i64,
    refresh_token_ttl_seconds: i64,
) -> AppResult<auth_service::AuthSession> {
    auth_service::bootstrap_admin(
        pool,
        request,
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
) -> AppResult<auth_service::AuthSession> {
    auth_service::refresh(
        pool,
        refresh_token,
        jwt_secret,
        access_token_ttl_seconds,
        refresh_token_ttl_seconds,
    )
    .await
}

pub async fn logout(pool: &PgPool, refresh_token: &str) -> AppResult<()> {
    auth_service::logout(pool, refresh_token).await
}

pub async fn change_password(
    pool: &PgPool,
    user_id: i64,
    request: ChangePasswordRequest,
) -> AppResult<()> {
    auth_service::change_password(pool, user_id, request).await
}

pub async fn find_user(pool: &PgPool, id: i64) -> AppResult<UserResponse> {
    auth_service::find_user(pool, id).await
}
