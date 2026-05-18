use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        CreateSupplierRequest, ListQuery, MessageResponse, SupplierResponse, UpdateSupplierRequest,
    },
};

use super::service;

pub async fn list_suppliers(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<SupplierResponse>>> {
    Ok(Json(service::list_suppliers(&state.db, query).await?))
}

pub async fn get_supplier(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<SupplierResponse>> {
    Ok(Json(service::find_supplier(&state.db, id).await?))
}

pub async fn create_supplier(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateSupplierRequest>,
) -> AppResult<Json<SupplierResponse>> {
    user.require_admin()?;
    Ok(Json(service::create_supplier(&state.db, payload).await?))
}

pub async fn update_supplier(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateSupplierRequest>,
) -> AppResult<Json<SupplierResponse>> {
    user.require_admin()?;
    Ok(Json(
        service::update_supplier(&state.db, id, payload).await?,
    ))
}

pub async fn delete_supplier(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<MessageResponse>> {
    user.require_admin()?;
    service::delete_supplier(&state.db, id).await?;
    Ok(Json(MessageResponse { message: "deleted" }))
}
