use sqlx::PgPool;

use crate::{
    auth::AuthUser,
    error::AppResult,
    models::{CancelRequest, CreateSaleRequest, ListQuery, SaleDetailResponse, SaleResponse},
    services::documents,
};

pub async fn list_sales(
    pool: &PgPool,
    user: &AuthUser,
    query: ListQuery,
) -> AppResult<Vec<SaleResponse>> {
    documents::list_sales(pool, user, query).await
}

pub async fn find_sale(pool: &PgPool, user: &AuthUser, id: i64) -> AppResult<SaleDetailResponse> {
    documents::find_sale(pool, user, id).await
}

pub async fn create_sale(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateSaleRequest,
) -> AppResult<SaleDetailResponse> {
    documents::create_sale(pool, user, request).await
}

pub async fn cancel_sale(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<SaleDetailResponse> {
    documents::cancel_sale(pool, user, id, request).await
}
