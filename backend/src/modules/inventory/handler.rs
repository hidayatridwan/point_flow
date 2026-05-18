use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        CancelRequest, CreateStockAdjustmentRequest, ListQuery, StockAdjustmentDetailResponse,
        StockAdjustmentResponse,
    },
};

use super::service;

pub async fn list_stock_adjustments(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<StockAdjustmentResponse>>> {
    Ok(Json(
        service::list_stock_adjustments(&state.db, query).await?,
    ))
}

pub async fn get_stock_adjustment(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<StockAdjustmentDetailResponse>> {
    Ok(Json(service::find_stock_adjustment(&state.db, id).await?))
}

pub async fn create_stock_adjustment(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateStockAdjustmentRequest>,
) -> AppResult<Json<StockAdjustmentDetailResponse>> {
    user.require_admin()?;
    Ok(Json(
        service::create_stock_adjustment(&state.db, &user, payload).await?,
    ))
}

pub async fn post_stock_adjustment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<StockAdjustmentDetailResponse>> {
    user.require_admin()?;
    Ok(Json(
        service::post_stock_adjustment(&state.db, &user, id).await?,
    ))
}

pub async fn cancel_stock_adjustment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CancelRequest>,
) -> AppResult<Json<StockAdjustmentDetailResponse>> {
    user.require_admin()?;
    Ok(Json(
        service::cancel_stock_adjustment(&state.db, &user, id, payload).await?,
    ))
}
