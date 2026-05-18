use sqlx::PgPool;

use crate::{
    error::AppResult,
    models::{CreateUserRequest, ListQuery, UpdateUserRequest, UserResponse},
    services::master,
};

pub async fn list_users(pool: &PgPool, query: ListQuery) -> AppResult<Vec<UserResponse>> {
    master::list_users(pool, query).await
}

pub async fn find_user(pool: &PgPool, id: i64) -> AppResult<UserResponse> {
    master::find_user(pool, id).await
}

pub async fn create_user(pool: &PgPool, request: CreateUserRequest) -> AppResult<UserResponse> {
    master::create_user(pool, request).await
}

pub async fn update_user(
    pool: &PgPool,
    id: i64,
    request: UpdateUserRequest,
) -> AppResult<UserResponse> {
    master::update_user(pool, id, request).await
}

pub async fn delete_user(pool: &PgPool, id: i64) -> AppResult<()> {
    master::delete_user(pool, id).await
}
