use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        ApiResponse, CreateRoleRequest, ListQuery, MessageResponse, PermissionResponse,
        RoleResponse, UpdateRolePermissionsRequest, UpdateRoleRequest,
    },
};

use super::service;

pub async fn list_roles(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<RoleResponse>>>> {
    user.require_permission(&state.db, "users").await?;
    Ok(Json(ApiResponse::success(
        service::list_roles(&state.db, query).await?,
    )))
}

pub async fn get_role(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<RoleResponse>>> {
    user.require_permission(&state.db, "users").await?;
    Ok(Json(ApiResponse::success(
        service::find_role(&state.db, id).await?,
    )))
}

pub async fn create_role(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateRoleRequest>,
) -> AppResult<Json<ApiResponse<RoleResponse>>> {
    user.require_permission(&state.db, "users").await?;
    Ok(Json(ApiResponse::success(
        service::create_role(&state.db, payload).await?,
    )))
}

pub async fn update_role(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateRoleRequest>,
) -> AppResult<Json<ApiResponse<RoleResponse>>> {
    user.require_permission(&state.db, "users").await?;
    Ok(Json(ApiResponse::success(
        service::update_role(&state.db, id, payload).await?,
    )))
}

pub async fn delete_role(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    user.require_permission(&state.db, "users").await?;
    service::delete_role(&state.db, id).await?;
    Ok(Json(ApiResponse::success(MessageResponse {
        message: "deleted",
    })))
}

pub async fn list_permissions(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<PermissionResponse>>>> {
    user.require_permission(&state.db, "users").await?;
    Ok(Json(ApiResponse::success(
        service::list_permissions(&state.db, query).await?,
    )))
}

pub async fn replace_role_permissions(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateRolePermissionsRequest>,
) -> AppResult<Json<ApiResponse<RoleResponse>>> {
    user.require_permission(&state.db, "users").await?;
    Ok(Json(ApiResponse::success(
        service::replace_role_permissions(&state.db, id, payload).await?,
    )))
}
