use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        ApiResponse, CancelRequest, CreateGoodsReceiptRequest, CreatePurchaseOrderRequest,
        GoodsReceiptDetailResponse, GoodsReceiptResponse, ListQuery, PurchaseOrderDetailResponse,
        PurchaseOrderResponse,
    },
};

use super::service;

pub async fn list_purchase_orders(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<PurchaseOrderResponse>>>> {
    Ok(Json(ApiResponse::success(
        service::list_purchase_orders(&state.db, query).await?,
    )))
}

pub async fn get_purchase_order(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<PurchaseOrderDetailResponse>>> {
    Ok(Json(ApiResponse::success(
        service::find_purchase_order(&state.db, id).await?,
    )))
}

pub async fn create_purchase_order(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreatePurchaseOrderRequest>,
) -> AppResult<Json<ApiResponse<PurchaseOrderDetailResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::create_purchase_order(&state.db, &user, payload).await?,
    )))
}

pub async fn order_purchase_order(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<PurchaseOrderDetailResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::order_purchase_order(&state.db, &user, id).await?,
    )))
}

pub async fn cancel_purchase_order(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CancelRequest>,
) -> AppResult<Json<ApiResponse<PurchaseOrderDetailResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::cancel_purchase_order(&state.db, &user, id, payload).await?,
    )))
}

pub async fn list_goods_receipts(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<GoodsReceiptResponse>>>> {
    Ok(Json(ApiResponse::success(
        service::list_goods_receipts(&state.db, query).await?,
    )))
}

pub async fn get_goods_receipt(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<GoodsReceiptDetailResponse>>> {
    Ok(Json(ApiResponse::success(
        service::find_goods_receipt(&state.db, id).await?,
    )))
}

pub async fn create_goods_receipt(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateGoodsReceiptRequest>,
) -> AppResult<Json<ApiResponse<GoodsReceiptDetailResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::create_goods_receipt(&state.db, &user, payload).await?,
    )))
}

pub async fn post_goods_receipt(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<GoodsReceiptDetailResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::post_goods_receipt(&state.db, &user, id).await?,
    )))
}

pub async fn cancel_goods_receipt(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CancelRequest>,
) -> AppResult<Json<ApiResponse<GoodsReceiptDetailResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::cancel_goods_receipt(&state.db, &user, id, payload).await?,
    )))
}
