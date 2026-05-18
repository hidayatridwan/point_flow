use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        CancelRequest, CreateGoodsReceiptRequest, CreatePurchaseOrderRequest,
        GoodsReceiptDetailResponse, GoodsReceiptResponse, ListQuery, PurchaseOrderDetailResponse,
        PurchaseOrderResponse,
    },
};

use super::service;

pub async fn list_purchase_orders(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<PurchaseOrderResponse>>> {
    Ok(Json(service::list_purchase_orders(&state.db, query).await?))
}

pub async fn get_purchase_order(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<PurchaseOrderDetailResponse>> {
    Ok(Json(service::find_purchase_order(&state.db, id).await?))
}

pub async fn create_purchase_order(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreatePurchaseOrderRequest>,
) -> AppResult<Json<PurchaseOrderDetailResponse>> {
    user.require_admin()?;
    Ok(Json(
        service::create_purchase_order(&state.db, &user, payload).await?,
    ))
}

pub async fn order_purchase_order(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<PurchaseOrderDetailResponse>> {
    user.require_admin()?;
    Ok(Json(
        service::order_purchase_order(&state.db, &user, id).await?,
    ))
}

pub async fn cancel_purchase_order(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CancelRequest>,
) -> AppResult<Json<PurchaseOrderDetailResponse>> {
    user.require_admin()?;
    Ok(Json(
        service::cancel_purchase_order(&state.db, &user, id, payload).await?,
    ))
}

pub async fn list_goods_receipts(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<GoodsReceiptResponse>>> {
    Ok(Json(service::list_goods_receipts(&state.db, query).await?))
}

pub async fn get_goods_receipt(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<GoodsReceiptDetailResponse>> {
    Ok(Json(service::find_goods_receipt(&state.db, id).await?))
}

pub async fn create_goods_receipt(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateGoodsReceiptRequest>,
) -> AppResult<Json<GoodsReceiptDetailResponse>> {
    user.require_admin()?;
    Ok(Json(
        service::create_goods_receipt(&state.db, &user, payload).await?,
    ))
}

pub async fn post_goods_receipt(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<GoodsReceiptDetailResponse>> {
    user.require_admin()?;
    Ok(Json(
        service::post_goods_receipt(&state.db, &user, id).await?,
    ))
}

pub async fn cancel_goods_receipt(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CancelRequest>,
) -> AppResult<Json<GoodsReceiptDetailResponse>> {
    user.require_admin()?;
    Ok(Json(
        service::cancel_goods_receipt(&state.db, &user, id, payload).await?,
    ))
}
