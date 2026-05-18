use sqlx::PgPool;

use crate::{
    error::AppResult,
    models::{
        DailySalesReport, DashboardResponse, GoodsReceiptReport, ListQuery, MonthlySalesReport,
        PayrollReport, PurchaseOrderReport, StockAdjustmentReport, StockReport,
    },
};

pub async fn dashboard(pool: &PgPool) -> AppResult<DashboardResponse> {
    sqlx::query_as::<_, DashboardResponse>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM sales WHERE status = 'completed' AND sold_at::date = CURRENT_DATE) AS transaction_count_today,
            (SELECT COALESCE(SUM(total_amount), 0)::float8 FROM sales WHERE status = 'completed' AND sold_at::date = CURRENT_DATE) AS revenue_today,
            (SELECT COUNT(*) FROM products WHERE deleted_at IS NULL AND is_active = TRUE AND stock <= minimum_stock) AS low_stock_product_count,
            (SELECT COUNT(*) FROM purchase_orders WHERE status IN ('ordered', 'partially_received')) AS pending_purchase_order_count,
            (SELECT COUNT(*) FROM stock_adjustments WHERE status = 'draft') AS draft_stock_adjustment_count,
            (SELECT COUNT(*) FROM employees WHERE deleted_at IS NULL AND is_active = TRUE) AS active_employee_count
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn daily_sales(pool: &PgPool, query: ListQuery) -> AppResult<Vec<DailySalesReport>> {
    sqlx::query_as::<_, DailySalesReport>(
        r#"
        SELECT
            sales_date,
            transaction_count,
            subtotal_amount::float8 AS subtotal_amount,
            total_amount::float8 AS total_amount
        FROM v_daily_sales_report
        WHERE ($1::date IS NULL OR sales_date >= $1)
            AND ($2::date IS NULL OR sales_date <= $2)
        ORDER BY sales_date DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(query.from)
    .bind(query.to)
    .bind(query.limit())
    .bind(query.offset())
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn monthly_sales(pool: &PgPool, query: ListQuery) -> AppResult<Vec<MonthlySalesReport>> {
    sqlx::query_as::<_, MonthlySalesReport>(
        r#"
        SELECT
            sales_month,
            transaction_count,
            subtotal_amount::float8 AS subtotal_amount,
            total_amount::float8 AS total_amount
        FROM v_monthly_sales_report
        WHERE ($1::date IS NULL OR sales_month >= date_trunc('month', $1::date)::date)
            AND ($2::date IS NULL OR sales_month <= date_trunc('month', $2::date)::date)
        ORDER BY sales_month DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(query.from)
    .bind(query.to)
    .bind(query.limit())
    .bind(query.offset())
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn stock(pool: &PgPool, query: ListQuery) -> AppResult<Vec<StockReport>> {
    let search = query.search_pattern();
    sqlx::query_as::<_, StockReport>(
        r#"
        SELECT
            id,
            name,
            sku,
            category_name,
            purchase_price::float8 AS purchase_price,
            selling_price::float8 AS selling_price,
            stock::float8 AS stock,
            minimum_stock::float8 AS minimum_stock,
            unit,
            quantity_on_order::float8 AS quantity_on_order,
            is_low_stock,
            is_active
        FROM v_stock_report
        WHERE $1 = '%%' OR lower(name) LIKE $1 OR lower(sku) LIKE $1
        ORDER BY is_low_stock DESC, name
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(search)
    .bind(query.limit())
    .bind(query.offset())
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn purchase_orders(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<PurchaseOrderReport>> {
    let search = query.search_pattern();
    sqlx::query_as::<_, PurchaseOrderReport>(
        r#"
        SELECT
            id,
            po_no,
            order_date,
            expected_date,
            supplier_name,
            status::text AS status,
            item_count,
            total_quantity_ordered::float8 AS total_quantity_ordered,
            total_quantity_received::float8 AS total_quantity_received,
            total_quantity_remaining::float8 AS total_quantity_remaining,
            subtotal_amount::float8 AS subtotal_amount,
            created_at,
            updated_at
        FROM v_purchase_order_report
        WHERE ($1 = '%%' OR lower(po_no) LIKE $1 OR lower(supplier_name) LIKE $1)
            AND ($2::date IS NULL OR order_date >= $2)
            AND ($3::date IS NULL OR order_date <= $3)
        ORDER BY id DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(search)
    .bind(query.from)
    .bind(query.to)
    .bind(query.limit())
    .bind(query.offset())
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn goods_receipts(pool: &PgPool, query: ListQuery) -> AppResult<Vec<GoodsReceiptReport>> {
    let search = query.search_pattern();
    sqlx::query_as::<_, GoodsReceiptReport>(
        r#"
        SELECT
            id,
            gr_no,
            received_at,
            status::text AS status,
            po_no,
            supplier_name,
            item_count,
            total_quantity_received::float8 AS total_quantity_received,
            total_amount::float8 AS total_amount,
            created_at,
            updated_at
        FROM v_goods_receipt_report
        WHERE ($1 = '%%' OR lower(gr_no) LIKE $1 OR lower(po_no) LIKE $1 OR lower(supplier_name) LIKE $1)
            AND ($2::date IS NULL OR received_at::date >= $2)
            AND ($3::date IS NULL OR received_at::date <= $3)
        ORDER BY id DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(search)
    .bind(query.from)
    .bind(query.to)
    .bind(query.limit())
    .bind(query.offset())
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn stock_adjustments(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<StockAdjustmentReport>> {
    let search = query.search_pattern();
    sqlx::query_as::<_, StockAdjustmentReport>(
        r#"
        SELECT
            id,
            adjustment_no,
            adjusted_at,
            status::text AS status,
            reason,
            item_count,
            total_quantity_increased::float8 AS total_quantity_increased,
            total_quantity_decreased::float8 AS total_quantity_decreased,
            created_by,
            posted_at,
            posted_by,
            created_at,
            updated_at
        FROM v_stock_adjustment_report
        WHERE ($1 = '%%' OR lower(adjustment_no) LIKE $1 OR lower(reason) LIKE $1)
            AND ($2::date IS NULL OR adjusted_at::date >= $2)
            AND ($3::date IS NULL OR adjusted_at::date <= $3)
        ORDER BY id DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(search)
    .bind(query.from)
    .bind(query.to)
    .bind(query.limit())
    .bind(query.offset())
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn payroll(pool: &PgPool, query: ListQuery) -> AppResult<Vec<PayrollReport>> {
    sqlx::query_as::<_, PayrollReport>(
        r#"
        SELECT
            period_start,
            period_end,
            slip_count,
            total_base_salary::float8 AS total_base_salary,
            total_bonus::float8 AS total_bonus,
            total_deduction::float8 AS total_deduction,
            total_net_salary::float8 AS total_net_salary
        FROM v_payroll_report
        WHERE ($1::date IS NULL OR period_start >= $1)
            AND ($2::date IS NULL OR period_end <= $2)
        ORDER BY period_start DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(query.from)
    .bind(query.to)
    .bind(query.limit())
    .bind(query.offset())
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}
