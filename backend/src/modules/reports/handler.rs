use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        DailySalesReport, DashboardResponse, GoodsReceiptReport, ListQuery, MonthlySalesReport,
        PayrollReport, PurchaseOrderReport, StockAdjustmentReport, StockReport,
    },
};

use super::service;

pub async fn dashboard(
    State(state): State<AppState>,
    _user: AuthUser,
) -> AppResult<Json<DashboardResponse>> {
    Ok(Json(service::dashboard(&state.db).await?))
}

pub async fn daily_sales(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<DailySalesReport>>> {
    Ok(Json(service::daily_sales(&state.db, query).await?))
}

pub async fn monthly_sales(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<MonthlySalesReport>>> {
    Ok(Json(service::monthly_sales(&state.db, query).await?))
}

pub async fn stock(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<StockReport>>> {
    Ok(Json(service::stock(&state.db, query).await?))
}

pub async fn purchase_orders(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<PurchaseOrderReport>>> {
    Ok(Json(service::purchase_orders(&state.db, query).await?))
}

pub async fn goods_receipts(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<GoodsReceiptReport>>> {
    Ok(Json(service::goods_receipts(&state.db, query).await?))
}

pub async fn stock_adjustments(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<StockAdjustmentReport>>> {
    Ok(Json(service::stock_adjustments(&state.db, query).await?))
}

pub async fn payroll(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<PayrollReport>>> {
    Ok(Json(service::payroll(&state.db, query).await?))
}
