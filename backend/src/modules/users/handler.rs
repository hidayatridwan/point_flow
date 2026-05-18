use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{CreateUserRequest, ListQuery, MessageResponse, UpdateUserRequest, UserResponse},
};

use super::service;

pub async fn list_users(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<UserResponse>>> {
    Ok(Json(service::list_users(&state.db, query).await?))
}

pub async fn get_user(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<UserResponse>> {
    Ok(Json(service::find_user(&state.db, id).await?))
}

pub async fn create_user(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    user.require_admin()?;
    Ok(Json(service::create_user(&state.db, payload).await?))
}

pub async fn update_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    user.require_admin()?;
    Ok(Json(service::update_user(&state.db, id, payload).await?))
}

pub async fn delete_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<MessageResponse>> {
    user.require_admin()?;
    service::delete_user(&state.db, id).await?;
    Ok(Json(MessageResponse { message: "deleted" }))
}
