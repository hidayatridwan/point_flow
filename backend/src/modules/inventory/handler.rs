use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        ApiResponse, CancelRequest, CreateStockAdjustmentRequest, CreateStockConversionRequest,
        ListQuery, StockAdjustmentDetailResponse, StockAdjustmentResponse,
        StockConversionDetailResponse, StockConversionResponse,
    },
};

use super::service;

pub async fn list_stock_adjustments(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<StockAdjustmentResponse>>>> {
    Ok(Json(ApiResponse::success(
        service::list_stock_adjustments(&state.db, query).await?,
    )))
}

pub async fn get_stock_adjustment(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<StockAdjustmentDetailResponse>>> {
    Ok(Json(ApiResponse::success(
        service::find_stock_adjustment(&state.db, id).await?,
    )))
}

pub async fn create_stock_adjustment(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateStockAdjustmentRequest>,
) -> AppResult<Json<ApiResponse<StockAdjustmentDetailResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::create_stock_adjustment(&state.db, &user, payload).await?,
    )))
}

pub async fn post_stock_adjustment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<StockAdjustmentDetailResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::post_stock_adjustment(&state.db, &user, id).await?,
    )))
}

pub async fn cancel_stock_adjustment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CancelRequest>,
) -> AppResult<Json<ApiResponse<StockAdjustmentDetailResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::cancel_stock_adjustment(&state.db, &user, id, payload).await?,
    )))
}

pub async fn list_stock_conversions(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<StockConversionResponse>>>> {
    Ok(Json(ApiResponse::success(
        service::list_stock_conversions(&state.db, query).await?,
    )))
}

pub async fn get_stock_conversion(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<StockConversionDetailResponse>>> {
    Ok(Json(ApiResponse::success(
        service::find_stock_conversion(&state.db, id).await?,
    )))
}

pub async fn create_stock_conversion(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateStockConversionRequest>,
) -> AppResult<Json<ApiResponse<StockConversionDetailResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::create_stock_conversion(&state.db, &user, payload).await?,
    )))
}

pub async fn post_stock_conversion(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<StockConversionDetailResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::post_stock_conversion(&state.db, &user, id).await?,
    )))
}

pub async fn cancel_stock_conversion(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CancelRequest>,
) -> AppResult<Json<ApiResponse<StockConversionDetailResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::cancel_stock_conversion(&state.db, &user, id, payload).await?,
    )))
}
