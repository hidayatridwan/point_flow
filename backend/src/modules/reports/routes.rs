use axum::{Router, routing::get};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(handler::dashboard))
        .route("/reports/daily-sales", get(handler::daily_sales))
        .route("/reports/monthly-sales", get(handler::monthly_sales))
        .route("/reports/stock", get(handler::stock))
        .route("/reports/purchase-orders", get(handler::purchase_orders))
        .route("/reports/goods-receipts", get(handler::goods_receipts))
        .route(
            "/reports/stock-adjustments",
            get(handler::stock_adjustments),
        )
        .route("/reports/payroll", get(handler::payroll))
}
