use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        ApiResponse, CancelRequest, CreateSaleReturnRequest, ListQuery, ReturnableSaleItemResponse,
        SaleReturnDetailResponse, SaleReturnResponse,
    },
};

use super::service;

pub async fn list_sale_returns(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<SaleReturnResponse>>>> {
    user.require_permission(&state.db, "pos").await?;
    Ok(Json(ApiResponse::success(
        service::list_sale_returns(&state.db, &user, query).await?,
    )))
}

pub async fn get_sale_return(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<SaleReturnDetailResponse>>> {
    user.require_permission(&state.db, "pos").await?;
    Ok(Json(ApiResponse::success(
        service::find_sale_return(&state.db, &user, id).await?,
    )))
}

pub async fn list_returnable_sale_items(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<Vec<ReturnableSaleItemResponse>>>> {
    user.require_permission(&state.db, "pos").await?;
    Ok(Json(ApiResponse::success(
        service::list_returnable_sale_items(&state.db, &user, id).await?,
    )))
}

pub async fn create_sale_return(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateSaleReturnRequest>,
) -> AppResult<Json<ApiResponse<SaleReturnDetailResponse>>> {
    user.require_permission(&state.db, "pos").await?;
    Ok(Json(ApiResponse::success(
        service::create_sale_return(&state.db, &user, payload).await?,
    )))
}

pub async fn cancel_sale_return(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CancelRequest>,
) -> AppResult<Json<ApiResponse<SaleReturnDetailResponse>>> {
    user.require_permission(&state.db, "pos").await?;
    Ok(Json(ApiResponse::success(
        service::cancel_sale_return(&state.db, &user, id, payload).await?,
    )))
}
