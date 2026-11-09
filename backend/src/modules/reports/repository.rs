use chrono::NaiveDate;
use sqlx::PgPool;

use crate::{
    error::AppResult,
    models::{
        CashFlowResponse, CashierPerformance, DailySalesReport, DashboardResponse,
        FinancialPositionResponse, GoodsReceiptReport, HourlySalesPoint, IncomeStatementResponse,
        ListQuery, MonthlySalesReport, PayrollReport, ProcurementFunnel, ProductScatterItem,
        PurchaseOrderReport, SalesByCategoryItem, SalesTrendPoint, StockAdjustmentReport,
        StockMovementReport, StockMovementReportQuery, StockReport, TopProductItem,
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

pub async fn stock_movements(
    pool: &PgPool,
    query: StockMovementReportQuery,
) -> AppResult<Vec<StockMovementReport>> {
    reports::stock_movements(pool, query).await
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

pub async fn sales_trend(pool: &PgPool, days: i32) -> AppResult<Vec<SalesTrendPoint>> {
    reports::sales_trend(pool, days).await
}

pub async fn top_products(pool: &PgPool, days: i32, limit: i32) -> AppResult<Vec<TopProductItem>> {
    reports::top_products(pool, days, limit).await
}

pub async fn hourly_sales(pool: &PgPool, date: NaiveDate) -> AppResult<Vec<HourlySalesPoint>> {
    reports::hourly_sales(pool, date).await
}

pub async fn low_stock(pool: &PgPool, limit: i64) -> AppResult<Vec<StockReport>> {
    reports::low_stock(pool, limit).await
}

pub async fn sales_by_category(pool: &PgPool, days: i32) -> AppResult<Vec<SalesByCategoryItem>> {
    reports::sales_by_category(pool, days).await
}

pub async fn product_scatter(
    pool: &PgPool,
    days: i32,
    limit: i32,
) -> AppResult<Vec<ProductScatterItem>> {
    reports::product_scatter(pool, days, limit).await
}

pub async fn cashier_performance(pool: &PgPool, days: i32) -> AppResult<Vec<CashierPerformance>> {
    reports::cashier_performance(pool, days).await
}

pub async fn procurement_funnel(pool: &PgPool) -> AppResult<ProcurementFunnel> {
    reports::procurement_funnel(pool).await
}

pub async fn income_statement(
    pool: &PgPool,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> AppResult<IncomeStatementResponse> {
    reports::income_statement(pool, from, to).await
}

pub async fn cash_flow(
    pool: &PgPool,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> AppResult<CashFlowResponse> {
    reports::cash_flow(pool, from, to).await
}

pub async fn financial_position(
    pool: &PgPool,
    as_of: Option<NaiveDate>,
) -> AppResult<FinancialPositionResponse> {
    reports::financial_position(pool, as_of).await
}
