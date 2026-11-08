use axum::{
    Json,
    extract::{Query, State},
};
use chrono::Local;

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        ApiResponse, CashierPerformance, DailySalesReport, DashboardResponse, GoodsReceiptReport,
        HourlySalesPoint, ListQuery, MonthlySalesReport, PayrollReport, ProcurementFunnel,
        ProductScatterItem, PurchaseOrderReport, SalesByCategoryItem, SalesTrendPoint,
        StockAdjustmentReport, StockMovementReport, StockMovementReportQuery, StockReport,
        TopProductItem,
    },
};

use super::{
    dto::{
        CashierPerformanceQuery, HourlySalesQuery, LowStockQuery, ProductScatterQuery,
        SalesByCategoryQuery, SalesTrendQuery, TopProductsQuery,
    },
    service,
};

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

pub async fn sales_trend(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<SalesTrendQuery>,
) -> AppResult<Json<ApiResponse<Vec<SalesTrendPoint>>>> {
    user.require_permission(&state.db, "dashboard").await?;
    let days = query.days.unwrap_or(30).clamp(1, 365);
    Ok(Json(ApiResponse::success(
        service::sales_trend(&state.db, days).await?,
    )))
}

pub async fn top_products(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<TopProductsQuery>,
) -> AppResult<Json<ApiResponse<Vec<TopProductItem>>>> {
    user.require_permission(&state.db, "dashboard").await?;
    let days = query.days.unwrap_or(30).clamp(1, 365);
    let limit = query.limit.unwrap_or(10).clamp(1, 20);
    Ok(Json(ApiResponse::success(
        service::top_products(&state.db, days, limit).await?,
    )))
}

pub async fn hourly_sales(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<HourlySalesQuery>,
) -> AppResult<Json<ApiResponse<Vec<HourlySalesPoint>>>> {
    user.require_permission(&state.db, "dashboard").await?;
    let date = query.date.unwrap_or_else(|| Local::now().date_naive());
    Ok(Json(ApiResponse::success(
        service::hourly_sales(&state.db, date).await?,
    )))
}

pub async fn low_stock(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<LowStockQuery>,
) -> AppResult<Json<ApiResponse<Vec<StockReport>>>> {
    user.require_permission(&state.db, "dashboard").await?;
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    Ok(Json(ApiResponse::success(
        service::low_stock(&state.db, limit).await?,
    )))
}

pub async fn sales_by_category(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<SalesByCategoryQuery>,
) -> AppResult<Json<ApiResponse<Vec<SalesByCategoryItem>>>> {
    user.require_permission(&state.db, "dashboard").await?;
    let days = query.days.unwrap_or(30).clamp(1, 365);
    Ok(Json(ApiResponse::success(
        service::sales_by_category(&state.db, days).await?,
    )))
}

pub async fn product_scatter(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ProductScatterQuery>,
) -> AppResult<Json<ApiResponse<Vec<ProductScatterItem>>>> {
    user.require_permission(&state.db, "dashboard").await?;
    let days = query.days.unwrap_or(30).clamp(1, 365);
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    Ok(Json(ApiResponse::success(
        service::product_scatter(&state.db, days, limit).await?,
    )))
}

pub async fn cashier_performance(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<CashierPerformanceQuery>,
) -> AppResult<Json<ApiResponse<Vec<CashierPerformance>>>> {
    user.require_permission(&state.db, "dashboard").await?;
    let days = query.days.unwrap_or(30).clamp(1, 365);
    Ok(Json(ApiResponse::success(
        service::cashier_performance(&state.db, days).await?,
    )))
}

pub async fn procurement_funnel(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<ProcurementFunnel>>> {
    user.require_permission(&state.db, "dashboard").await?;
    Ok(Json(ApiResponse::success(
        service::procurement_funnel(&state.db).await?,
    )))
}
