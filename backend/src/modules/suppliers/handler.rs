use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        ApiResponse, CreateSupplierRequest, ListQuery, MessageResponse, SupplierResponse,
        UpdateSupplierRequest,
    },
};

use super::service;

pub async fn list_suppliers(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<SupplierResponse>>>> {
    Ok(Json(ApiResponse::success(
        service::list_suppliers(&state.db, query).await?,
    )))
}

pub async fn get_supplier(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<SupplierResponse>>> {
    Ok(Json(ApiResponse::success(
        service::find_supplier(&state.db, id).await?,
    )))
}

pub async fn create_supplier(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateSupplierRequest>,
) -> AppResult<Json<ApiResponse<SupplierResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::create_supplier(&state.db, payload).await?,
    )))
}

pub async fn update_supplier(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateSupplierRequest>,
) -> AppResult<Json<ApiResponse<SupplierResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::update_supplier(&state.db, id, payload).await?,
    )))
}

pub async fn delete_supplier(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    user.require_admin()?;
    service::delete_supplier(&state.db, id).await?;
    Ok(Json(ApiResponse::success(MessageResponse {
        message: "deleted",
    })))
}
