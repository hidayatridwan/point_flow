use sqlx::PgPool;

use crate::{
    auth::AuthUser,
    error::AppResult,
    models::{
        CancelRequest, CreateGoodsReceiptRequest, CreatePurchaseOrderRequest,
        GoodsReceiptDetailResponse, GoodsReceiptResponse, ListQuery, PurchaseOrderDetailResponse,
        PurchaseOrderResponse,
    },
    services::documents,
};

pub async fn list_purchase_orders(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<PurchaseOrderResponse>> {
    documents::list_purchase_orders(pool, query).await
}

pub async fn find_purchase_order(pool: &PgPool, id: i64) -> AppResult<PurchaseOrderDetailResponse> {
    documents::find_purchase_order(pool, id).await
}

pub async fn create_purchase_order(
    pool: &PgPool,
    user: &AuthUser,
    request: CreatePurchaseOrderRequest,
) -> AppResult<PurchaseOrderDetailResponse> {
    documents::create_purchase_order(pool, user, request).await
}

pub async fn order_purchase_order(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
) -> AppResult<PurchaseOrderDetailResponse> {
    documents::order_purchase_order(pool, user, id).await
}

pub async fn cancel_purchase_order(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<PurchaseOrderDetailResponse> {
    documents::cancel_purchase_order(pool, user, id, request).await
}

pub async fn list_goods_receipts(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<GoodsReceiptResponse>> {
    documents::list_goods_receipts(pool, query).await
}

pub async fn find_goods_receipt(pool: &PgPool, id: i64) -> AppResult<GoodsReceiptDetailResponse> {
    documents::find_goods_receipt(pool, id).await
}

pub async fn create_goods_receipt(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateGoodsReceiptRequest,
) -> AppResult<GoodsReceiptDetailResponse> {
    documents::create_goods_receipt(pool, user, request).await
}

pub async fn post_goods_receipt(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
) -> AppResult<GoodsReceiptDetailResponse> {
    documents::post_goods_receipt(pool, user, id).await
}

pub async fn cancel_goods_receipt(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<GoodsReceiptDetailResponse> {
    documents::cancel_goods_receipt(pool, user, id, request).await
}
