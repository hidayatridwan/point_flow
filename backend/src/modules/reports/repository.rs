use sqlx::PgPool;

use crate::{
    error::AppResult,
    models::{
        DailySalesReport, DashboardResponse, GoodsReceiptReport, ListQuery, MonthlySalesReport,
        PayrollReport, PurchaseOrderReport, StockAdjustmentReport, StockReport,
    },
    services::reports,
};

pub async fn dashboard(pool: &PgPool) -> AppResult<DashboardResponse> {
    reports::dashboard(pool).await
}

pub async fn daily_sales(pool: &PgPool, query: ListQuery) -> AppResult<Vec<DailySalesReport>> {
    reports::daily_sales(pool, query).await
}

pub async fn monthly_sales(pool: &PgPool, query: ListQuery) -> AppResult<Vec<MonthlySalesReport>> {
    reports::monthly_sales(pool, query).await
}

pub async fn stock(pool: &PgPool, query: ListQuery) -> AppResult<Vec<StockReport>> {
    reports::stock(pool, query).await
}

pub async fn purchase_orders(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<PurchaseOrderReport>> {
    reports::purchase_orders(pool, query).await
}

pub async fn goods_receipts(pool: &PgPool, query: ListQuery) -> AppResult<Vec<GoodsReceiptReport>> {
    reports::goods_receipts(pool, query).await
}

pub async fn stock_adjustments(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<StockAdjustmentReport>> {
    reports::stock_adjustments(pool, query).await
}

pub async fn payroll(pool: &PgPool, query: ListQuery) -> AppResult<Vec<PayrollReport>> {
    reports::payroll(pool, query).await
}
