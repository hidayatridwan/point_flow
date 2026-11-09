use axum::{Router, routing::get};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(handler::dashboard))
        .route("/dashboard/sales-trend", get(handler::sales_trend))
        .route("/dashboard/top-products", get(handler::top_products))
        .route("/dashboard/hourly-sales", get(handler::hourly_sales))
        .route("/dashboard/low-stock", get(handler::low_stock))
        .route(
            "/dashboard/sales-by-category",
            get(handler::sales_by_category),
        )
        .route("/dashboard/product-scatter", get(handler::product_scatter))
        .route(
            "/dashboard/cashier-performance",
            get(handler::cashier_performance),
        )
        .route(
            "/dashboard/procurement-funnel",
            get(handler::procurement_funnel),
        )
        .route("/reports/daily-sales", get(handler::daily_sales))
        .route("/reports/monthly-sales", get(handler::monthly_sales))
        .route("/reports/stock", get(handler::stock))
        .route("/reports/stock-movements", get(handler::stock_movements))
        .route("/reports/purchase-orders", get(handler::purchase_orders))
        .route("/reports/goods-receipts", get(handler::goods_receipts))
        .route(
            "/reports/stock-adjustments",
            get(handler::stock_adjustments),
        )
        .route("/reports/payroll", get(handler::payroll))
        .route("/reports/income-statement", get(handler::income_statement))
        .route("/reports/cash-flow", get(handler::cash_flow))
        .route(
            "/reports/financial-position",
            get(handler::financial_position),
        )
}
