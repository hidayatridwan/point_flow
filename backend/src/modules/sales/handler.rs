use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        ApiResponse, CancelRequest, CreateSaleRequest, ListQuery, SaleDetailResponse, SaleResponse,
    },
};

use super::service;

pub async fn list_sales(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<SaleResponse>>>> {
    Ok(Json(ApiResponse::success(
        service::list_sales(&state.db, &user, query).await?,
    )))
}

pub async fn get_sale(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<SaleDetailResponse>>> {
    Ok(Json(ApiResponse::success(
        service::find_sale(&state.db, &user, id).await?,
    )))
}

pub async fn create_sale(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateSaleRequest>,
) -> AppResult<Json<ApiResponse<SaleDetailResponse>>> {
    Ok(Json(ApiResponse::success(
        service::create_sale(&state.db, &user, payload).await?,
    )))
}

pub async fn cancel_sale(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CancelRequest>,
) -> AppResult<Json<ApiResponse<SaleDetailResponse>>> {
    Ok(Json(ApiResponse::success(
        service::cancel_sale(&state.db, &user, id, payload).await?,
    )))
}
