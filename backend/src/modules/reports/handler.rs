use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        ApiResponse, DailySalesReport, DashboardResponse, GoodsReceiptReport, ListQuery,
        MonthlySalesReport, PayrollReport, PurchaseOrderReport, StockAdjustmentReport,
        StockMovementReport, StockMovementReportQuery, StockReport,
    },
};

use super::service;

pub async fn dashboard(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<DashboardResponse>>> {
    user.require_permission(&state.db, "dashboard").await?;
    Ok(Json(ApiResponse::success(
        service::dashboard(&state.db).await?,
    )))
}

pub async fn daily_sales(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<DailySalesReport>>>> {
    user.require_permission(&state.db, "reports").await?;
    Ok(Json(ApiResponse::success(
        service::daily_sales(&state.db, query).await?,
    )))
}

pub async fn monthly_sales(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<MonthlySalesReport>>>> {
    user.require_permission(&state.db, "reports").await?;
    Ok(Json(ApiResponse::success(
        service::monthly_sales(&state.db, query).await?,
    )))
}

pub async fn stock(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<StockReport>>>> {
    user.require_any_permission(&state.db, &["inventory", "reports"])
        .await?;
    Ok(Json(ApiResponse::success(
        service::stock(&state.db, query).await?,
    )))
}

pub async fn stock_movements(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<StockMovementReportQuery>,
) -> AppResult<Json<ApiResponse<Vec<StockMovementReport>>>> {
    user.require_any_permission(&state.db, &["inventory", "reports"])
        .await?;
    Ok(Json(ApiResponse::success(
        service::stock_movements(&state.db, query).await?,
    )))
}

pub async fn purchase_orders(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<PurchaseOrderReport>>>> {
    user.require_permission(&state.db, "reports").await?;
    Ok(Json(ApiResponse::success(
        service::purchase_orders(&state.db, query).await?,
    )))
}

pub async fn goods_receipts(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<GoodsReceiptReport>>>> {
    user.require_permission(&state.db, "reports").await?;
    Ok(Json(ApiResponse::success(
        service::goods_receipts(&state.db, query).await?,
    )))
}

pub async fn stock_adjustments(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<StockAdjustmentReport>>>> {
    user.require_permission(&state.db, "reports").await?;
    Ok(Json(ApiResponse::success(
        service::stock_adjustments(&state.db, query).await?,
    )))
}

pub async fn payroll(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<PayrollReport>>>> {
    user.require_any_permission(&state.db, &["payroll", "reports"])
        .await?;
    Ok(Json(ApiResponse::success(
        service::payroll(&state.db, query).await?,
    )))
}
