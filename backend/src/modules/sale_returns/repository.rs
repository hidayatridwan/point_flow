use sqlx::PgPool;

use crate::{
    auth::AuthUser,
    error::AppResult,
    models::{
        CancelRequest, CreateSaleReturnRequest, ListQuery, ReturnableSaleItemResponse,
        SaleReturnDetailResponse, SaleReturnResponse,
    },
    services::documents,
};

pub async fn list_sale_returns(
    pool: &PgPool,
    user: &AuthUser,
    query: ListQuery,
) -> AppResult<Vec<SaleReturnResponse>> {
    documents::list_sale_returns(pool, user, query).await
}

pub async fn find_sale_return(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
) -> AppResult<SaleReturnDetailResponse> {
    documents::find_sale_return(pool, user, id).await
}

pub async fn list_returnable_sale_items(
    pool: &PgPool,
    user: &AuthUser,
    sale_id: i64,
) -> AppResult<Vec<ReturnableSaleItemResponse>> {
    documents::list_returnable_sale_items(pool, user, sale_id).await
}

pub async fn create_sale_return(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateSaleReturnRequest,
) -> AppResult<SaleReturnDetailResponse> {
    documents::create_sale_return(pool, user, request).await
}

pub async fn cancel_sale_return(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<SaleReturnDetailResponse> {
    documents::cancel_sale_return(pool, user, id, request).await
}
