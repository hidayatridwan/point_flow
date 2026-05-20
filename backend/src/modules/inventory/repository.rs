use sqlx::PgPool;

use crate::{
    auth::AuthUser,
    error::AppResult,
    models::{
        CancelRequest, CreateStockAdjustmentRequest, CreateStockConversionRequest, ListQuery,
        StockAdjustmentDetailResponse, StockAdjustmentResponse, StockConversionDetailResponse,
        StockConversionResponse,
    },
    services::documents,
};

pub async fn list_stock_adjustments(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<StockAdjustmentResponse>> {
    documents::list_stock_adjustments(pool, query).await
}

pub async fn find_stock_adjustment(
    pool: &PgPool,
    id: i64,
) -> AppResult<StockAdjustmentDetailResponse> {
    documents::find_stock_adjustment(pool, id).await
}

pub async fn create_stock_adjustment(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateStockAdjustmentRequest,
) -> AppResult<StockAdjustmentDetailResponse> {
    documents::create_stock_adjustment(pool, user, request).await
}

pub async fn post_stock_adjustment(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
) -> AppResult<StockAdjustmentDetailResponse> {
    documents::post_stock_adjustment(pool, user, id).await
}

pub async fn cancel_stock_adjustment(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<StockAdjustmentDetailResponse> {
    documents::cancel_stock_adjustment(pool, user, id, request).await
}

pub async fn list_stock_conversions(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<StockConversionResponse>> {
    documents::list_stock_conversions(pool, query).await
}

pub async fn find_stock_conversion(
    pool: &PgPool,
    id: i64,
) -> AppResult<StockConversionDetailResponse> {
    documents::find_stock_conversion(pool, id).await
}

pub async fn create_stock_conversion(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateStockConversionRequest,
) -> AppResult<StockConversionDetailResponse> {
    documents::create_stock_conversion(pool, user, request).await
}

pub async fn post_stock_conversion(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
) -> AppResult<StockConversionDetailResponse> {
    documents::post_stock_conversion(pool, user, id).await
}

pub async fn cancel_stock_conversion(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<StockConversionDetailResponse> {
    documents::cancel_stock_conversion(pool, user, id, request).await
}
