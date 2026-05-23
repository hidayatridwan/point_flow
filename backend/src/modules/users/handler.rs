use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        ApiResponse, CreateUserRequest, ListQuery, MessageResponse, UpdateUserRequest, UserResponse,
    },
};

use super::service;

pub async fn list_users(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<UserResponse>>>> {
    user.require_permission(&state.db, "users").await?;
    Ok(Json(ApiResponse::success(
        service::list_users(&state.db, query).await?,
    )))
}

pub async fn get_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    user.require_permission(&state.db, "users").await?;
    Ok(Json(ApiResponse::success(
        service::find_user(&state.db, id).await?,
    )))
}

pub async fn create_user(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    user.require_permission(&state.db, "users").await?;
    Ok(Json(ApiResponse::success(
        service::create_user(&state.db, payload).await?,
    )))
}

pub async fn update_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    user.require_permission(&state.db, "users").await?;
    Ok(Json(ApiResponse::success(
        service::update_user(&state.db, id, payload).await?,
    )))
}

pub async fn delete_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    user.require_permission(&state.db, "users").await?;
    service::delete_user(&state.db, id).await?;
    Ok(Json(ApiResponse::success(MessageResponse {
        message: "deleted",
    })))
}
