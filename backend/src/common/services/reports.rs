use chrono::NaiveDate;
use sqlx::PgPool;

use crate::{
    error::{AppError, AppResult},
    models::{
        CashFlowResponse, CashierPerformance, DailySalesReport, DashboardResponse,
        FinancialPositionResponse, GoodsReceiptReport, HourlySalesPoint, IncomeStatementResponse,
        ListQuery, MonthlySalesReport, PayrollReport, ProcurementFunnel, ProductScatterItem,
        PurchaseOrderReport, SalesByCategoryItem, SalesTrendPoint, StockAdjustmentReport,
        StockMovementReport, StockMovementReportQuery, StockReport, TopProductItem,
    },
};

use super::{
    GOODS_RECEIPT_STATUSES, PURCHASE_ORDER_STATUSES, STOCK_ADJUSTMENT_STATUSES,
    STOCK_MOVEMENT_TYPES, STOCK_REPORT_STATUSES, parse_status_filter,
};

pub async fn dashboard(pool: &PgPool) -> AppResult<DashboardResponse> {
    sqlx::query_as::<_, DashboardResponse>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM sales WHERE status = 'completed' AND sold_at::date = CURRENT_DATE) AS transaction_count_today,
            (SELECT COALESCE(SUM(total_amount), 0)::float8 FROM sales WHERE status = 'completed' AND sold_at::date = CURRENT_DATE) AS revenue_today,
            (SELECT COUNT(*) FROM product_skus ps JOIN products p ON p.id = ps.product_id WHERE ps.deleted_at IS NULL AND ps.is_active = TRUE AND ps.stock <= ps.minimum_stock AND p.deleted_at IS NULL AND p.is_active = TRUE) AS low_stock_product_count,
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
    let statuses = parse_status_filter(query.status.as_deref(), STOCK_REPORT_STATUSES, "stock")?;
    sqlx::query_as::<_, StockReport>(
        r#"
        SELECT
            product_sku_id,
            product_id,
            product_name,
            sku,
            barcode,
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
        WHERE ($1 = '%%' OR lower(product_name) LIKE $1 OR lower(sku) LIKE $1 OR lower(coalesce(barcode, '')) LIKE $1)
            AND (
                COALESCE(array_length($2::text[], 1), 0) = 0
                OR ('low_stock' = ANY($2::text[]) AND is_active = TRUE AND is_low_stock = TRUE)
                OR ('in_stock' = ANY($2::text[]) AND is_active = TRUE AND is_low_stock = FALSE)
                OR ('inactive' = ANY($2::text[]) AND is_active = FALSE)
            )
        ORDER BY is_low_stock DESC, product_name, sku
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(search)
    .bind(statuses)
    .bind(query.limit())
    .bind(query.offset())
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn stock_movements(
    pool: &PgPool,
    query: StockMovementReportQuery,
) -> AppResult<Vec<StockMovementReport>> {
    let search = query.search_pattern();
    let movement_types = parse_status_filter(
        query.movement_type.as_deref(),
        STOCK_MOVEMENT_TYPES,
        "stock movement",
    )?;
    let product_sku_id = parse_optional_i64(query.product_sku_id.as_deref(), "product_sku_id")?;

    sqlx::query_as::<_, StockMovementReport>(
        r#"
        SELECT
            sm.id,
            sm.created_at,
            sm.product_id,
            sm.product_sku_id,
            p.name AS product_name,
            ps.sku,
            u.code AS unit,
            sm.movement_type::text AS movement_type,
            CASE
                WHEN sm.stock_after >= sm.stock_before THEN 'in'
                ELSE 'out'
            END AS direction,
            sm.quantity::float8 AS quantity,
            sm.stock_before::float8 AS stock_before,
            sm.stock_after::float8 AS stock_after,
            (sm.stock_after - sm.stock_before)::float8 AS quantity_delta,
            CASE
                WHEN sm.sale_id IS NOT NULL THEN 'sale'
                WHEN sm.goods_receipt_id IS NOT NULL THEN 'goods_receipt'
                WHEN sm.stock_adjustment_id IS NOT NULL THEN 'stock_adjustment'
                WHEN sm.stock_conversion_id IS NOT NULL THEN 'stock_conversion'
                ELSE 'unknown'
            END AS reference_type,
            CASE
                WHEN sm.sale_id IS NOT NULL THEN sm.sale_id
                WHEN sm.goods_receipt_id IS NOT NULL THEN sm.goods_receipt_id
                WHEN sm.stock_adjustment_id IS NOT NULL THEN sm.stock_adjustment_id
                WHEN sm.stock_conversion_id IS NOT NULL THEN sm.stock_conversion_id
                ELSE NULL
            END AS reference_id,
            CASE
                WHEN sm.sale_id IS NOT NULL THEN s.transaction_no
                WHEN sm.goods_receipt_id IS NOT NULL THEN gr.gr_no
                WHEN sm.stock_adjustment_id IS NOT NULL THEN sa.adjustment_no
                WHEN sm.stock_conversion_id IS NOT NULL THEN sc.conversion_no
                ELSE NULL
            END AS reference_no,
            sm.notes,
            sm.created_by
        FROM stock_movements sm
        JOIN products p ON p.id = sm.product_id
        JOIN product_skus ps ON ps.id = sm.product_sku_id
        JOIN units u ON u.id = ps.unit_id
        LEFT JOIN sales s ON s.id = sm.sale_id
        LEFT JOIN goods_receipts gr ON gr.id = sm.goods_receipt_id
        LEFT JOIN stock_adjustments sa ON sa.id = sm.stock_adjustment_id
        LEFT JOIN stock_conversions sc ON sc.id = sm.stock_conversion_id
        WHERE (
                $1 = '%%'
                OR lower(p.name) LIKE $1
                OR lower(ps.sku) LIKE $1
                OR lower(coalesce(ps.barcode, '')) LIKE $1
                OR lower(coalesce(s.transaction_no, '')) LIKE $1
                OR lower(coalesce(gr.gr_no, '')) LIKE $1
                OR lower(coalesce(sa.adjustment_no, '')) LIKE $1
                OR lower(coalesce(sc.conversion_no, '')) LIKE $1
                OR lower(coalesce(sm.notes, '')) LIKE $1
            )
            AND ($2::date IS NULL OR sm.created_at::date >= $2)
            AND ($3::date IS NULL OR sm.created_at::date <= $3)
            AND (COALESCE(array_length($4::text[], 1), 0) = 0 OR sm.movement_type::text = ANY($4::text[]))
            AND ($5::bigint IS NULL OR sm.product_sku_id = $5)
        ORDER BY sm.created_at DESC, sm.id DESC
        LIMIT $6 OFFSET $7
        "#,
    )
    .bind(search)
    .bind(query.from)
    .bind(query.to)
    .bind(movement_types)
    .bind(product_sku_id)
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
    let statuses = parse_status_filter(
        query.status.as_deref(),
        PURCHASE_ORDER_STATUSES,
        "purchase order",
    )?;
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
            AND (COALESCE(array_length($4::text[], 1), 0) = 0 OR status::text = ANY($4::text[]))
        ORDER BY id DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(search)
    .bind(query.from)
    .bind(query.to)
    .bind(statuses)
    .bind(query.limit())
    .bind(query.offset())
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

fn parse_optional_i64(value: Option<&str>, field: &str) -> AppResult<Option<i64>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    value
        .parse::<i64>()
        .map(Some)
        .map_err(|_| AppError::BadRequest(format!("{field} must be a valid integer")))
}

pub async fn goods_receipts(pool: &PgPool, query: ListQuery) -> AppResult<Vec<GoodsReceiptReport>> {
    let search = query.search_pattern();
    let statuses = parse_status_filter(
        query.status.as_deref(),
        GOODS_RECEIPT_STATUSES,
        "goods receipt",
    )?;
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
            AND (COALESCE(array_length($4::text[], 1), 0) = 0 OR status::text = ANY($4::text[]))
        ORDER BY id DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(search)
    .bind(query.from)
    .bind(query.to)
    .bind(statuses)
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
    let statuses = parse_status_filter(
        query.status.as_deref(),
        STOCK_ADJUSTMENT_STATUSES,
        "stock adjustment",
    )?;
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
            AND (COALESCE(array_length($4::text[], 1), 0) = 0 OR status::text = ANY($4::text[]))
        ORDER BY id DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(search)
    .bind(query.from)
    .bind(query.to)
    .bind(statuses)
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

pub async fn sales_trend(pool: &PgPool, days: i32) -> AppResult<Vec<SalesTrendPoint>> {
    let days = days.clamp(1, 365);
    sqlx::query_as::<_, SalesTrendPoint>(
        r#"
        SELECT
            d.date::date AS date,
            COALESCE(SUM(s.total_amount), 0)::float8 AS revenue,
            COUNT(s.id) AS transaction_count
        FROM generate_series(
            CURRENT_DATE - ($1::int - 1) * INTERVAL '1 day',
            CURRENT_DATE,
            INTERVAL '1 day'
        ) AS d(date)
        LEFT JOIN sales s
            ON s.sold_at::date = d.date::date
            AND s.status = 'completed'
        GROUP BY d.date
        ORDER BY d.date ASC
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn top_products(pool: &PgPool, days: i32, limit: i32) -> AppResult<Vec<TopProductItem>> {
    let days = days.clamp(1, 365);
    let limit = limit.clamp(1, 20);
    sqlx::query_as::<_, TopProductItem>(
        r#"
        SELECT
            si.product_sku_id,
            MIN(si.product_name) AS product_name,
            MIN(si.product_sku)  AS product_sku,
            MIN(si.unit)         AS unit,
            COALESCE(SUM(si.line_total), 0)::float8 AS revenue,
            COALESCE(SUM(si.quantity),   0)::float8 AS quantity_sold
        FROM sale_items si
        JOIN sales s ON s.id = si.sale_id
        WHERE s.status = 'completed'
            AND s.sold_at >= CURRENT_TIMESTAMP - ($1::int * INTERVAL '1 day')
        GROUP BY si.product_sku_id
        ORDER BY revenue DESC
        LIMIT $2
        "#,
    )
    .bind(days)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn hourly_sales(pool: &PgPool, date: NaiveDate) -> AppResult<Vec<HourlySalesPoint>> {
    sqlx::query_as::<_, HourlySalesPoint>(
        r#"
        SELECT
            h.hour::int AS hour,
            COALESCE(SUM(s.total_amount), 0)::float8 AS revenue,
            COUNT(s.id) AS transaction_count
        FROM generate_series(0, 23) AS h(hour)
        LEFT JOIN sales s
            ON EXTRACT(HOUR FROM s.sold_at)::int = h.hour
            AND s.sold_at::date = $1
            AND s.status = 'completed'
        GROUP BY h.hour
        ORDER BY h.hour ASC
        "#,
    )
    .bind(date)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn low_stock(pool: &PgPool, limit: i64) -> AppResult<Vec<StockReport>> {
    let limit = limit.clamp(1, 100);
    sqlx::query_as::<_, StockReport>(
        r#"
        SELECT
            product_sku_id,
            product_id,
            product_name,
            sku,
            barcode,
            category_name,
            purchase_price::float8,
            selling_price::float8,
            stock::float8,
            minimum_stock::float8,
            unit,
            quantity_on_order::float8,
            is_low_stock,
            is_active
        FROM v_stock_report
        WHERE is_low_stock = TRUE AND is_active = TRUE
        ORDER BY (stock - minimum_stock) ASC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn sales_by_category(pool: &PgPool, days: i32) -> AppResult<Vec<SalesByCategoryItem>> {
    let days = days.clamp(1, 365);
    sqlx::query_as::<_, SalesByCategoryItem>(
        r#"
        SELECT
            COALESCE(c.name, 'Uncategorized') AS category_name,
            COALESCE(SUM(si.line_total), 0)::float8 AS revenue,
            COUNT(DISTINCT s.id) AS transaction_count,
            ROUND(
                (COALESCE(SUM(si.line_total), 0) /
                 NULLIF(SUM(SUM(si.line_total)) OVER (), 0)) * 100, 2
            )::float8 AS percentage
        FROM sale_items si
        JOIN sales s ON s.id = si.sale_id
        JOIN products p ON p.id = si.product_id
        LEFT JOIN product_categories c ON c.id = p.category_id
        WHERE s.status = 'completed'
            AND s.sold_at >= CURRENT_TIMESTAMP - ($1::int * INTERVAL '1 day')
        GROUP BY c.id, c.name
        ORDER BY revenue DESC
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn product_scatter(
    pool: &PgPool,
    days: i32,
    limit: i32,
) -> AppResult<Vec<ProductScatterItem>> {
    let days = days.clamp(1, 365);
    let limit = limit.clamp(1, 100);
    sqlx::query_as::<_, ProductScatterItem>(
        r#"
        SELECT
            si.product_sku_id,
            MIN(si.product_name)          AS product_name,
            MIN(si.product_sku)           AS product_sku,
            MIN(si.unit)                  AS unit,
            MIN(ps.selling_price)::float8 AS selling_price,
            COALESCE(SUM(si.quantity),   0)::float8 AS quantity_sold,
            COALESCE(SUM(si.line_total), 0)::float8 AS revenue
        FROM sale_items si
        JOIN sales s         ON s.id  = si.sale_id
        JOIN product_skus ps ON ps.id = si.product_sku_id
        WHERE s.status = 'completed'
            AND s.sold_at >= CURRENT_TIMESTAMP - ($1::int * INTERVAL '1 day')
        GROUP BY si.product_sku_id
        ORDER BY quantity_sold DESC
        LIMIT $2
        "#,
    )
    .bind(days)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn cashier_performance(pool: &PgPool, days: i32) -> AppResult<Vec<CashierPerformance>> {
    let days = days.clamp(1, 365);
    sqlx::query_as::<_, CashierPerformance>(
        r#"
        WITH sales_agg AS (
            SELECT
                cashier_id,
                COUNT(*)                        AS transaction_count,
                COALESCE(SUM(total_amount), 0)  AS revenue,
                COALESCE(AVG(total_amount), 0)  AS avg_basket_size
            FROM sales
            WHERE status = 'completed'
                AND sold_at >= CURRENT_TIMESTAMP - ($1::int * INTERVAL '1 day')
            GROUP BY cashier_id
        ),
        returns_agg AS (
            SELECT cashier_id, COUNT(*) AS return_count
            FROM sale_returns
            WHERE status = 'completed'
                AND returned_at >= CURRENT_TIMESTAMP - ($1::int * INTERVAL '1 day')
            GROUP BY cashier_id
        )
        SELECT
            u.id            AS cashier_id,
            u.name          AS cashier_name,
            sa.transaction_count,
            sa.revenue::float8,
            sa.avg_basket_size::float8,
            COALESCE(ra.return_count, 0) AS return_count
        FROM users u
        JOIN sales_agg sa        ON sa.cashier_id = u.id
        LEFT JOIN returns_agg ra ON ra.cashier_id = u.id
        WHERE u.deleted_at IS NULL
        ORDER BY revenue DESC
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Laba rugi derivatif (desain Bagian 2.2). HPP memakai snapshot
/// `sale_items.unit_cost`, fallback ke harga beli berjalan untuk penjualan
/// pra-migrasi (`unit_cost = 0`). Retur mengurangi pendapatan dan HPP
/// proporsional berdasarkan tanggal retur.
pub async fn income_statement(
    pool: &PgPool,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> AppResult<IncomeStatementResponse> {
    sqlx::query_as::<_, IncomeStatementResponse>(
        r#"
        WITH base AS (
            SELECT
                (SELECT COALESCE(SUM(total_amount), 0)
                 FROM sales
                 WHERE status = 'completed'
                     AND ($1::date IS NULL OR sold_at::date >= $1)
                     AND ($2::date IS NULL OR sold_at::date <= $2)) AS gross_revenue,
                (SELECT COALESCE(SUM(subtotal_amount), 0)
                 FROM sale_returns
                 WHERE status = 'completed'
                     AND ($1::date IS NULL OR returned_at::date >= $1)
                     AND ($2::date IS NULL OR returned_at::date <= $2)) AS return_amount,
                (SELECT COALESCE(SUM(si.quantity * COALESCE(NULLIF(si.unit_cost, 0), ps.purchase_price)), 0)
                 FROM sale_items si
                 JOIN sales s ON s.id = si.sale_id
                 JOIN product_skus ps ON ps.id = si.product_sku_id
                 WHERE s.status = 'completed'
                     AND ($1::date IS NULL OR s.sold_at::date >= $1)
                     AND ($2::date IS NULL OR s.sold_at::date <= $2)) AS cogs_gross,
                (SELECT COALESCE(SUM(sri.quantity * COALESCE(NULLIF(si.unit_cost, 0), ps.purchase_price)), 0)
                 FROM sale_return_items sri
                 JOIN sale_returns sr ON sr.id = sri.sale_return_id
                 JOIN sale_items si ON si.id = sri.sale_item_id
                 JOIN product_skus ps ON ps.id = si.product_sku_id
                 WHERE sr.status = 'completed'
                     AND ($1::date IS NULL OR sr.returned_at::date >= $1)
                     AND ($2::date IS NULL OR sr.returned_at::date <= $2)) AS cogs_returned,
                (SELECT COALESCE(SUM(net_salary), 0)
                 FROM payroll_slips
                 WHERE ($1::date IS NULL OR period_start >= $1)
                     AND ($2::date IS NULL OR period_end <= $2)) AS payroll_expense,
                (SELECT COALESCE(SUM(amount), 0)
                 FROM financial_transactions
                 WHERE type = 'expense'
                     AND status = 'completed'
                     AND ($1::date IS NULL OR occurred_at::date >= $1)
                     AND ($2::date IS NULL OR occurred_at::date <= $2)) AS other_expense
        )
        SELECT
            gross_revenue::float8 AS gross_revenue,
            return_amount::float8 AS return_amount,
            (gross_revenue - return_amount)::float8 AS net_revenue,
            (cogs_gross - cogs_returned)::float8 AS cogs,
            (gross_revenue - return_amount - (cogs_gross - cogs_returned))::float8 AS gross_profit,
            payroll_expense::float8 AS payroll_expense,
            other_expense::float8 AS other_expense,
            (gross_revenue - return_amount - (cogs_gross - cogs_returned) - payroll_expense - other_expense)::float8 AS net_profit
        FROM base
        "#,
    )
    .bind(from)
    .bind(to)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

/// Arus kas derivatif (desain Bagian 2.2). Hanya transaksi tunai
/// (`is_cash = TRUE`) yang dihitung; pergerakan drawer `cash_in` bukan kas keluar.
pub async fn cash_flow(
    pool: &PgPool,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> AppResult<CashFlowResponse> {
    sqlx::query_as::<_, CashFlowResponse>(
        r#"
        WITH base AS (
            SELECT
                (SELECT COALESCE(SUM(total_amount), 0)
                 FROM sales
                 WHERE status = 'completed'
                     AND ($1::date IS NULL OR sold_at::date >= $1)
                     AND ($2::date IS NULL OR sold_at::date <= $2)) AS cash_in_sales,
                (SELECT COALESCE(SUM(amount), 0)
                 FROM financial_transactions
                 WHERE type = 'income'
                     AND status = 'completed'
                     AND is_cash = TRUE
                     AND ($1::date IS NULL OR occurred_at::date >= $1)
                     AND ($2::date IS NULL OR occurred_at::date <= $2)) AS cash_in_manual,
                (SELECT COALESCE(SUM(amount), 0)
                 FROM cash_drawer_movements
                 WHERE movement_type IN ('cash_out', 'cash_drop', 'petty_cash')
                     AND status = 'completed'
                     AND ($1::date IS NULL OR occurred_at::date >= $1)
                     AND ($2::date IS NULL OR occurred_at::date <= $2)) AS cash_out_drawer,
                (SELECT COALESCE(SUM(amount), 0)
                 FROM financial_transactions
                 WHERE type = 'expense'
                     AND status = 'completed'
                     AND is_cash = TRUE
                     AND ($1::date IS NULL OR occurred_at::date >= $1)
                     AND ($2::date IS NULL OR occurred_at::date <= $2)) AS cash_out_manual
        )
        SELECT
            cash_in_sales::float8 AS cash_in_sales,
            cash_in_manual::float8 AS cash_in_manual,
            (cash_in_sales + cash_in_manual)::float8 AS total_cash_in,
            cash_out_drawer::float8 AS cash_out_drawer,
            cash_out_manual::float8 AS cash_out_manual,
            (cash_out_drawer + cash_out_manual)::float8 AS total_cash_out,
            ((cash_in_sales + cash_in_manual) - (cash_out_drawer + cash_out_manual))::float8 AS net_cash_flow
        FROM base
        "#,
    )
    .bind(from)
    .bind(to)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

/// Posisi keuangan sederhana (desain Bagian 2.2): kas dan laba dihitung
/// kumulatif dari transaksi pertama s/d `as_of` (default hari ini);
/// persediaan memakai snapshot stok × harga beli saat ini.
pub async fn financial_position(
    pool: &PgPool,
    as_of: Option<NaiveDate>,
) -> AppResult<FinancialPositionResponse> {
    sqlx::query_as::<_, FinancialPositionResponse>(
        r#"
        WITH base AS (
            SELECT
                COALESCE($1, CURRENT_DATE)::date AS cutoff,
                (SELECT COALESCE(SUM(total_amount), 0)
                 FROM sales
                 WHERE status = 'completed'
                     AND sold_at::date <= COALESCE($1, CURRENT_DATE)) AS cash_in_sales,
                (SELECT COALESCE(SUM(amount), 0)
                 FROM financial_transactions
                 WHERE type = 'income'
                     AND status = 'completed'
                     AND is_cash = TRUE
                     AND occurred_at::date <= COALESCE($1, CURRENT_DATE)) AS cash_in_manual,
                (SELECT COALESCE(SUM(amount), 0)
                 FROM cash_drawer_movements
                 WHERE movement_type IN ('cash_out', 'cash_drop', 'petty_cash')
                     AND status = 'completed'
                     AND occurred_at::date <= COALESCE($1, CURRENT_DATE)) AS cash_out_drawer,
                (SELECT COALESCE(SUM(amount), 0)
                 FROM financial_transactions
                 WHERE type = 'expense'
                     AND status = 'completed'
                     AND is_cash = TRUE
                     AND occurred_at::date <= COALESCE($1, CURRENT_DATE)) AS cash_out_manual,
                (SELECT COALESCE(SUM(stock * purchase_price), 0)
                 FROM v_stock_report) AS inventory_value,
                (SELECT COALESCE(SUM(total_amount), 0)
                 FROM sales
                 WHERE status = 'completed'
                     AND sold_at::date <= COALESCE($1, CURRENT_DATE)) AS gross_revenue,
                (SELECT COALESCE(SUM(subtotal_amount), 0)
                 FROM sale_returns
                 WHERE status = 'completed'
                     AND returned_at::date <= COALESCE($1, CURRENT_DATE)) AS return_amount,
                (SELECT COALESCE(SUM(si.quantity * COALESCE(NULLIF(si.unit_cost, 0), ps.purchase_price)), 0)
                 FROM sale_items si
                 JOIN sales s ON s.id = si.sale_id
                 JOIN product_skus ps ON ps.id = si.product_sku_id
                 WHERE s.status = 'completed'
                     AND s.sold_at::date <= COALESCE($1, CURRENT_DATE)) AS cogs_gross,
                (SELECT COALESCE(SUM(sri.quantity * COALESCE(NULLIF(si.unit_cost, 0), ps.purchase_price)), 0)
                 FROM sale_return_items sri
                 JOIN sale_returns sr ON sr.id = sri.sale_return_id
                 JOIN sale_items si ON si.id = sri.sale_item_id
                 JOIN product_skus ps ON ps.id = si.product_sku_id
                 WHERE sr.status = 'completed'
                     AND sr.returned_at::date <= COALESCE($1, CURRENT_DATE)) AS cogs_returned,
                (SELECT COALESCE(SUM(net_salary), 0)
                 FROM payroll_slips
                 WHERE period_end <= COALESCE($1, CURRENT_DATE)) AS payroll_expense,
                (SELECT COALESCE(SUM(amount), 0)
                 FROM financial_transactions
                 WHERE type = 'expense'
                     AND status = 'completed'
                     AND occurred_at::date <= COALESCE($1, CURRENT_DATE)) AS other_expense
        )
        SELECT
            cutoff AS as_of,
            ((cash_in_sales + cash_in_manual) - (cash_out_drawer + cash_out_manual))::float8 AS cash,
            inventory_value::float8 AS inventory_value,
            (((cash_in_sales + cash_in_manual) - (cash_out_drawer + cash_out_manual)) + inventory_value)::float8 AS total_assets,
            (gross_revenue - return_amount - (cogs_gross - cogs_returned) - payroll_expense - other_expense)::float8 AS cumulative_net_profit
        FROM base
        "#,
    )
    .bind(as_of)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn procurement_funnel(pool: &PgPool) -> AppResult<ProcurementFunnel> {
    sqlx::query_as::<_, ProcurementFunnel>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM purchase_orders WHERE status = 'draft')              AS po_draft,
            (SELECT COUNT(*) FROM purchase_orders WHERE status = 'ordered')            AS po_ordered,
            (SELECT COUNT(*) FROM purchase_orders WHERE status = 'partially_received') AS po_partially_received,
            (SELECT COUNT(*) FROM purchase_orders WHERE status = 'received')           AS po_received,
            (SELECT COUNT(*) FROM purchase_orders WHERE status = 'cancelled')          AS po_cancelled,
            (SELECT COUNT(*) FROM goods_receipts  WHERE status = 'draft')              AS gr_draft,
            (SELECT COUNT(*) FROM goods_receipts  WHERE status = 'posted')             AS gr_posted,
            (SELECT COUNT(*) FROM goods_receipts  WHERE status = 'cancelled')          AS gr_cancelled,
            (SELECT COUNT(*) FROM stock_adjustments WHERE status = 'draft')            AS sa_draft,
            (SELECT COUNT(*) FROM stock_adjustments WHERE status = 'posted')           AS sa_posted,
            (SELECT COUNT(*) FROM stock_adjustments WHERE status = 'cancelled')        AS sa_cancelled
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}
