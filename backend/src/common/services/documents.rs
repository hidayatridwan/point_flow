use chrono::Utc;
use sqlx::{PgPool, Postgres, Row, Transaction};

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::{
        CancelRequest, CreateGoodsReceiptRequest, CreatePayrollSlipRequest,
        CreatePurchaseOrderRequest, CreateSaleRequest, CreateStockAdjustmentRequest,
        CreateStockConversionRequest, GoodsReceiptDetailResponse, GoodsReceiptItemResponse,
        GoodsReceiptResponse, ListQuery, PayrollSlipResponse, PurchaseOrderDetailResponse,
        PurchaseOrderItemResponse, PurchaseOrderResponse, SaleDetailResponse, SaleItemResponse,
        SaleResponse, StockAdjustmentDetailResponse, StockAdjustmentItemResponse,
        StockAdjustmentResponse, StockConversionDetailResponse, StockConversionItemResponse,
        StockConversionResponse,
    },
};

use super::{
    GOODS_RECEIPT_STATUSES, PURCHASE_ORDER_STATUSES, SALE_STATUSES, STOCK_ADJUSTMENT_STATUSES,
    STOCK_CONVERSION_STATUSES, parse_status_filter, validate_non_negative, validate_positive,
    validate_required,
};

const PURCHASE_ORDER_SELECT: &str = r#"
    SELECT
        po.id,
        po.po_no,
        po.supplier_id,
        s.name AS supplier_name,
        po.order_date,
        po.expected_date,
        po.status::text AS status,
        po.subtotal_amount::float8 AS subtotal_amount,
        po.notes,
        po.ordered_by,
        po.ordered_at,
        po.cancelled_at,
        po.cancelled_by,
        po.cancel_reason,
        po.created_by,
        po.created_at,
        po.updated_at
    FROM purchase_orders po
    JOIN suppliers s ON s.id = po.supplier_id
"#;

const PURCHASE_ORDER_ITEM_SELECT: &str = r#"
    SELECT
        id,
        purchase_order_id,
        product_id,
        product_sku_id,
        product_name,
        product_sku,
        unit,
        quantity_ordered::float8 AS quantity_ordered,
        quantity_received::float8 AS quantity_received,
        unit_cost::float8 AS unit_cost,
        line_total::float8 AS line_total,
        created_at,
        updated_at
    FROM purchase_order_items
"#;

const GOODS_RECEIPT_SELECT: &str = r#"
    SELECT
        gr.id,
        gr.gr_no,
        gr.purchase_order_id,
        po.po_no,
        gr.received_at,
        gr.status::text AS status,
        gr.received_by,
        gr.posted_at,
        gr.posted_by,
        gr.cancelled_at,
        gr.cancelled_by,
        gr.cancel_reason,
        gr.notes,
        gr.created_at,
        gr.updated_at
    FROM goods_receipts gr
    JOIN purchase_orders po ON po.id = gr.purchase_order_id
"#;

const GOODS_RECEIPT_ITEM_SELECT: &str = r#"
    SELECT
        id,
        goods_receipt_id,
        purchase_order_item_id,
        product_id,
        product_sku_id,
        product_name,
        product_sku,
        unit,
        quantity_received::float8 AS quantity_received,
        unit_cost::float8 AS unit_cost,
        line_total::float8 AS line_total,
        created_at,
        updated_at
    FROM goods_receipt_items
"#;

const STOCK_ADJUSTMENT_SELECT: &str = r#"
    SELECT
        id,
        adjustment_no,
        adjusted_at,
        status::text AS status,
        reason,
        notes,
        created_by,
        posted_at,
        posted_by,
        cancelled_at,
        cancelled_by,
        cancel_reason,
        created_at,
        updated_at
    FROM stock_adjustments
"#;

const STOCK_ADJUSTMENT_ITEM_SELECT: &str = r#"
    SELECT
        id,
        stock_adjustment_id,
        product_id,
        product_sku_id,
        product_name,
        product_sku,
        unit,
        stock_before::float8 AS stock_before,
        stock_after::float8 AS stock_after,
        quantity_delta::float8 AS quantity_delta,
        notes,
        created_at,
        updated_at
    FROM stock_adjustment_items
"#;

const STOCK_CONVERSION_SELECT: &str = r#"
    SELECT
        id,
        conversion_no,
        converted_at,
        status::text AS status,
        notes,
        created_by,
        posted_at,
        posted_by,
        cancelled_at,
        cancelled_by,
        cancel_reason,
        created_at,
        updated_at
    FROM stock_conversions
"#;

const STOCK_CONVERSION_ITEM_SELECT: &str = r#"
    SELECT
        id,
        stock_conversion_id,
        from_product_id,
        from_product_sku_id,
        from_product_name,
        from_product_sku,
        from_unit,
        from_stock_before::float8 AS from_stock_before,
        from_quantity::float8 AS from_quantity,
        from_stock_after::float8 AS from_stock_after,
        to_product_id,
        to_product_sku_id,
        to_product_name,
        to_product_sku,
        to_unit,
        to_stock_before::float8 AS to_stock_before,
        to_quantity::float8 AS to_quantity,
        to_stock_after::float8 AS to_stock_after,
        notes,
        created_at,
        updated_at
    FROM stock_conversion_items
"#;

const SALE_SELECT: &str = r#"
    SELECT
        s.id,
        s.transaction_no,
        s.sold_at,
        s.cashier_id,
        u.name AS cashier_name,
        s.subtotal_amount::float8 AS subtotal_amount,
        s.total_amount::float8 AS total_amount,
        s.paid_amount::float8 AS paid_amount,
        s.change_amount::float8 AS change_amount,
        s.status::text AS status,
        s.notes,
        s.cancelled_at,
        s.cancelled_by,
        s.cancel_reason,
        s.created_at,
        s.updated_at
    FROM sales s
    JOIN users u ON u.id = s.cashier_id
"#;

const SALE_ITEM_SELECT: &str = r#"
    SELECT
        id,
        sale_id,
        product_id,
        product_sku_id,
        product_name,
        product_sku,
        unit,
        quantity::float8 AS quantity,
        unit_price::float8 AS unit_price,
        line_total::float8 AS line_total,
        created_at
    FROM sale_items
"#;

const PAYROLL_SELECT: &str = r#"
    SELECT
        id,
        slip_no,
        employee_id,
        employee_name,
        employee_position,
        period_start,
        period_end,
        base_salary::float8 AS base_salary,
        bonus::float8 AS bonus,
        deduction::float8 AS deduction,
        net_salary::float8 AS net_salary,
        created_by,
        created_at,
        updated_at
    FROM payroll_slips
"#;

pub async fn list_purchase_orders(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<PurchaseOrderResponse>> {
    let search = query.search_pattern();
    let statuses = parse_status_filter(
        query.status.as_deref(),
        PURCHASE_ORDER_STATUSES,
        "purchase order",
    )?;
    let sql = format!(
        "{PURCHASE_ORDER_SELECT}
        WHERE ($1 = '%%' OR lower(po.po_no) LIKE $1 OR lower(s.name) LIKE $1)
            AND ($2::date IS NULL OR po.order_date >= $2)
            AND ($3::date IS NULL OR po.order_date <= $3)
            AND (COALESCE(array_length($4::text[], 1), 0) = 0 OR po.status::text = ANY($4::text[]))
        ORDER BY po.id DESC
        LIMIT $5 OFFSET $6"
    );

    sqlx::query_as::<_, PurchaseOrderResponse>(&sql)
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

pub async fn find_purchase_order(pool: &PgPool, id: i64) -> AppResult<PurchaseOrderDetailResponse> {
    let header_sql = format!("{PURCHASE_ORDER_SELECT} WHERE po.id = $1");
    let items_sql =
        format!("{PURCHASE_ORDER_ITEM_SELECT} WHERE purchase_order_id = $1 ORDER BY id");

    let purchase_order = sqlx::query_as::<_, PurchaseOrderResponse>(&header_sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;
    let items = sqlx::query_as::<_, PurchaseOrderItemResponse>(&items_sql)
        .bind(id)
        .fetch_all(pool)
        .await?;

    Ok(PurchaseOrderDetailResponse {
        purchase_order,
        items,
    })
}

pub async fn create_purchase_order(
    pool: &PgPool,
    user: &AuthUser,
    request: CreatePurchaseOrderRequest,
) -> AppResult<PurchaseOrderDetailResponse> {
    if request.items.is_empty() {
        return Err(AppError::Validation("items must not be empty".to_string()));
    }
    if let Some(expected_date) = request.expected_date {
        let order_date = request
            .order_date
            .unwrap_or_else(|| Utc::now().date_naive());
        if expected_date < order_date {
            return Err(AppError::Validation(
                "expected_date must be greater than or equal to order_date".to_string(),
            ));
        }
    }

    let mut subtotal = 0.0;
    for item in &request.items {
        validate_positive(item.quantity_ordered, "quantity_ordered")?;
        validate_non_negative(item.unit_cost, "unit_cost")?;
        subtotal += item.quantity_ordered * item.unit_cost;
    }

    let mut tx = pool.begin().await?;
    let order_date = request
        .order_date
        .unwrap_or_else(|| Utc::now().date_naive());
    let po_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO purchase_orders (
            supplier_id, order_date, expected_date, subtotal_amount, notes, created_by
        )
        VALUES ($1, $2, $3, $4::numeric, $5, $6)
        RETURNING id
        "#,
    )
    .bind(request.supplier_id)
    .bind(order_date)
    .bind(request.expected_date)
    .bind(subtotal)
    .bind(request.notes)
    .bind(user.id)
    .fetch_one(&mut *tx)
    .await?;

    for item in request.items {
        let product = load_product_sku_snapshot(&mut tx, item.product_sku_id, false).await?;
        sqlx::query(
            r#"
            INSERT INTO purchase_order_items (
                purchase_order_id, product_id, product_sku_id, product_name, product_sku, unit,
                quantity_ordered, unit_cost
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8::numeric)
            "#,
        )
        .bind(po_id)
        .bind(product.product_id)
        .bind(product.id)
        .bind(product.product_name)
        .bind(product.sku)
        .bind(product.unit)
        .bind(item.quantity_ordered)
        .bind(item.unit_cost)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    find_purchase_order(pool, po_id).await
}

pub async fn order_purchase_order(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
) -> AppResult<PurchaseOrderDetailResponse> {
    let result = sqlx::query(
        r#"
        UPDATE purchase_orders
        SET status = 'ordered', ordered_by = $2, ordered_at = now()
        WHERE id = $1 AND status = 'draft'
        "#,
    )
    .bind(id)
    .bind(user.id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Conflict(
            "purchase order must be in draft status".to_string(),
        ));
    }

    find_purchase_order(pool, id).await
}

pub async fn cancel_purchase_order(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<PurchaseOrderDetailResponse> {
    let result = sqlx::query(
        r#"
        UPDATE purchase_orders
        SET status = 'cancelled', cancelled_at = now(), cancelled_by = $2, cancel_reason = $3
        WHERE id = $1 AND status IN ('draft', 'ordered', 'partially_received')
        "#,
    )
    .bind(id)
    .bind(user.id)
    .bind(request.reason)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Conflict(
            "purchase order cannot be cancelled".to_string(),
        ));
    }

    find_purchase_order(pool, id).await
}

pub async fn list_goods_receipts(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<GoodsReceiptResponse>> {
    let search = query.search_pattern();
    let statuses = parse_status_filter(
        query.status.as_deref(),
        GOODS_RECEIPT_STATUSES,
        "goods receipt",
    )?;
    let sql = format!(
        "{GOODS_RECEIPT_SELECT}
        WHERE ($1 = '%%' OR lower(gr.gr_no) LIKE $1 OR lower(po.po_no) LIKE $1)
            AND ($2::date IS NULL OR gr.received_at::date >= $2)
            AND ($3::date IS NULL OR gr.received_at::date <= $3)
            AND (COALESCE(array_length($4::text[], 1), 0) = 0 OR gr.status::text = ANY($4::text[]))
        ORDER BY gr.id DESC
        LIMIT $5 OFFSET $6"
    );

    sqlx::query_as::<_, GoodsReceiptResponse>(&sql)
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

pub async fn find_goods_receipt(pool: &PgPool, id: i64) -> AppResult<GoodsReceiptDetailResponse> {
    let header_sql = format!("{GOODS_RECEIPT_SELECT} WHERE gr.id = $1");
    let items_sql = format!("{GOODS_RECEIPT_ITEM_SELECT} WHERE goods_receipt_id = $1 ORDER BY id");

    let goods_receipt = sqlx::query_as::<_, GoodsReceiptResponse>(&header_sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;
    let items = sqlx::query_as::<_, GoodsReceiptItemResponse>(&items_sql)
        .bind(id)
        .fetch_all(pool)
        .await?;

    Ok(GoodsReceiptDetailResponse {
        goods_receipt,
        items,
    })
}

pub async fn create_goods_receipt(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateGoodsReceiptRequest,
) -> AppResult<GoodsReceiptDetailResponse> {
    if request.items.is_empty() {
        return Err(AppError::Validation("items must not be empty".to_string()));
    }
    for item in &request.items {
        validate_positive(item.quantity_received, "quantity_received")?;
        if let Some(unit_cost) = item.unit_cost {
            validate_non_negative(unit_cost, "unit_cost")?;
        }
    }

    let mut tx = pool.begin().await?;
    let po_status: String =
        sqlx::query_scalar("SELECT status::text FROM purchase_orders WHERE id = $1")
            .bind(request.purchase_order_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(AppError::NotFound)?;
    if !["ordered", "partially_received"].contains(&po_status.as_str()) {
        return Err(AppError::Conflict(
            "purchase order must be ordered before goods receipt can be created".to_string(),
        ));
    }

    let gr_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO goods_receipts (purchase_order_id, received_at, received_by, notes)
        VALUES ($1, COALESCE($2, now()), $3, $4)
        RETURNING id
        "#,
    )
    .bind(request.purchase_order_id)
    .bind(request.received_at)
    .bind(user.id)
    .bind(request.notes)
    .fetch_one(&mut *tx)
    .await?;

    for item in request.items {
        let po_item = load_purchase_order_item(&mut tx, item.purchase_order_item_id, false).await?;
        if po_item.purchase_order_id != request.purchase_order_id {
            return Err(AppError::Validation(
                "purchase_order_item_id does not belong to purchase_order_id".to_string(),
            ));
        }
        let remaining = po_item.quantity_ordered - po_item.quantity_received;
        if item.quantity_received > remaining {
            return Err(AppError::Validation(
                "quantity_received exceeds remaining purchase order quantity".to_string(),
            ));
        }
        let unit_cost = item.unit_cost.unwrap_or(po_item.unit_cost);

        sqlx::query(
            r#"
            INSERT INTO goods_receipt_items (
                goods_receipt_id, purchase_order_item_id, product_id, product_sku_id, product_name,
                product_sku, unit, quantity_received, unit_cost
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9::numeric)
            "#,
        )
        .bind(gr_id)
        .bind(po_item.id)
        .bind(po_item.product_id)
        .bind(po_item.product_sku_id)
        .bind(po_item.product_name)
        .bind(po_item.product_sku)
        .bind(po_item.unit)
        .bind(item.quantity_received)
        .bind(unit_cost)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    find_goods_receipt(pool, gr_id).await
}

pub async fn post_goods_receipt(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
) -> AppResult<GoodsReceiptDetailResponse> {
    let mut tx = pool.begin().await?;
    let receipt_status: String =
        sqlx::query_scalar("SELECT status::text FROM goods_receipts WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(AppError::NotFound)?;
    if receipt_status != "draft" {
        return Err(AppError::Conflict(
            "goods receipt must be in draft status".to_string(),
        ));
    }

    let items = load_goods_receipt_items(&mut tx, id).await?;
    if items.is_empty() {
        return Err(AppError::Validation("items must not be empty".to_string()));
    }

    let po_id: i64 =
        sqlx::query_scalar("SELECT purchase_order_id FROM goods_receipts WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

    for item in items {
        let po_item = load_purchase_order_item(&mut tx, item.purchase_order_item_id, true).await?;
        let remaining = po_item.quantity_ordered - po_item.quantity_received;
        if item.quantity_received > remaining {
            return Err(AppError::Validation(
                "quantity_received exceeds remaining purchase order quantity".to_string(),
            ));
        }

        let product = load_product_sku_snapshot(&mut tx, item.product_sku_id, true).await?;
        let stock_before = product.stock;
        let stock_after = stock_before + item.quantity_received;

        sqlx::query("UPDATE product_skus SET stock = $2::numeric WHERE id = $1")
            .bind(item.product_sku_id)
            .bind(stock_after)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "UPDATE purchase_order_items SET quantity_received = quantity_received + $2::numeric WHERE id = $1",
        )
        .bind(item.purchase_order_item_id)
        .bind(item.quantity_received)
        .execute(&mut *tx)
        .await?;
        insert_stock_movement(
            &mut tx,
            item.product_id,
            item.product_sku_id,
            "purchase_receipt",
            item.quantity_received,
            stock_before,
            stock_after,
            Some("Goods receipt posted"),
            Some(user.id),
            None,
            None,
            Some(id),
            Some(item.id),
            None,
            None,
            None,
            None,
        )
        .await?;
    }

    update_purchase_order_status(&mut tx, po_id).await?;
    sqlx::query(
        "UPDATE goods_receipts SET status = 'posted', posted_at = now(), posted_by = $2 WHERE id = $1",
    )
    .bind(id)
    .bind(user.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    find_goods_receipt(pool, id).await
}

pub async fn cancel_goods_receipt(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<GoodsReceiptDetailResponse> {
    let result = sqlx::query(
        r#"
        UPDATE goods_receipts
        SET status = 'cancelled', cancelled_at = now(), cancelled_by = $2, cancel_reason = $3
        WHERE id = $1 AND status = 'draft'
        "#,
    )
    .bind(id)
    .bind(user.id)
    .bind(request.reason)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Conflict(
            "only draft goods receipts can be cancelled".to_string(),
        ));
    }

    find_goods_receipt(pool, id).await
}

pub async fn list_stock_adjustments(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<StockAdjustmentResponse>> {
    let search = query.search_pattern();
    let statuses = parse_status_filter(
        query.status.as_deref(),
        STOCK_ADJUSTMENT_STATUSES,
        "stock adjustment",
    )?;
    let sql = format!(
        "{STOCK_ADJUSTMENT_SELECT}
        WHERE ($1 = '%%' OR lower(adjustment_no) LIKE $1 OR lower(reason) LIKE $1)
            AND ($2::date IS NULL OR adjusted_at::date >= $2)
            AND ($3::date IS NULL OR adjusted_at::date <= $3)
            AND (COALESCE(array_length($4::text[], 1), 0) = 0 OR status::text = ANY($4::text[]))
        ORDER BY id DESC
        LIMIT $5 OFFSET $6"
    );

    sqlx::query_as::<_, StockAdjustmentResponse>(&sql)
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

pub async fn find_stock_adjustment(
    pool: &PgPool,
    id: i64,
) -> AppResult<StockAdjustmentDetailResponse> {
    let header_sql = format!("{STOCK_ADJUSTMENT_SELECT} WHERE id = $1");
    let items_sql =
        format!("{STOCK_ADJUSTMENT_ITEM_SELECT} WHERE stock_adjustment_id = $1 ORDER BY id");

    let stock_adjustment = sqlx::query_as::<_, StockAdjustmentResponse>(&header_sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;
    let items = sqlx::query_as::<_, StockAdjustmentItemResponse>(&items_sql)
        .bind(id)
        .fetch_all(pool)
        .await?;

    Ok(StockAdjustmentDetailResponse {
        stock_adjustment,
        items,
    })
}

pub async fn create_stock_adjustment(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateStockAdjustmentRequest,
) -> AppResult<StockAdjustmentDetailResponse> {
    validate_required(&request.reason, "reason")?;
    if request.items.is_empty() {
        return Err(AppError::Validation("items must not be empty".to_string()));
    }
    for item in &request.items {
        validate_non_negative(item.stock_after, "stock_after")?;
    }

    let mut tx = pool.begin().await?;
    let adjustment_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO stock_adjustments (adjusted_at, reason, notes, created_by)
        VALUES (COALESCE($1, now()), $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(request.adjusted_at)
    .bind(request.reason.trim())
    .bind(request.notes)
    .bind(user.id)
    .fetch_one(&mut *tx)
    .await?;

    for item in request.items {
        let product = load_product_sku_snapshot(&mut tx, item.product_sku_id, false).await?;
        if (product.stock - item.stock_after).abs() < f64::EPSILON {
            return Err(AppError::Validation(
                "stock_after must be different from stock_before".to_string(),
            ));
        }
        sqlx::query(
            r#"
            INSERT INTO stock_adjustment_items (
                stock_adjustment_id, product_id, product_sku_id, product_name, product_sku,
                unit, stock_before, stock_after, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8::numeric, $9)
            "#,
        )
        .bind(adjustment_id)
        .bind(product.product_id)
        .bind(product.id)
        .bind(product.product_name)
        .bind(product.sku)
        .bind(product.unit)
        .bind(product.stock)
        .bind(item.stock_after)
        .bind(item.notes)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    find_stock_adjustment(pool, adjustment_id).await
}

pub async fn post_stock_adjustment(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
) -> AppResult<StockAdjustmentDetailResponse> {
    let mut tx = pool.begin().await?;
    let status: String =
        sqlx::query_scalar("SELECT status::text FROM stock_adjustments WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(AppError::NotFound)?;
    if status != "draft" {
        return Err(AppError::Conflict(
            "stock adjustment must be in draft status".to_string(),
        ));
    }

    let items = load_stock_adjustment_items(&mut tx, id).await?;
    if items.is_empty() {
        return Err(AppError::Validation("items must not be empty".to_string()));
    }

    for item in items {
        let product = load_product_sku_snapshot(&mut tx, item.product_sku_id, true).await?;
        if (product.stock - item.stock_before).abs() > f64::EPSILON {
            return Err(AppError::Conflict(format!(
                "product SKU {} stock changed after adjustment was drafted",
                item.product_sku
            )));
        }
        let quantity = item.quantity_delta.abs();
        sqlx::query("UPDATE product_skus SET stock = $2::numeric WHERE id = $1")
            .bind(item.product_sku_id)
            .bind(item.stock_after)
            .execute(&mut *tx)
            .await?;
        insert_stock_movement(
            &mut tx,
            item.product_id,
            item.product_sku_id,
            "adjustment",
            quantity,
            item.stock_before,
            item.stock_after,
            item.notes.as_deref(),
            Some(user.id),
            None,
            None,
            None,
            None,
            Some(id),
            Some(item.id),
            None,
            None,
        )
        .await?;
    }

    sqlx::query(
        "UPDATE stock_adjustments SET status = 'posted', posted_at = now(), posted_by = $2 WHERE id = $1",
    )
    .bind(id)
    .bind(user.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    find_stock_adjustment(pool, id).await
}

pub async fn cancel_stock_adjustment(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<StockAdjustmentDetailResponse> {
    let result = sqlx::query(
        r#"
        UPDATE stock_adjustments
        SET status = 'cancelled', cancelled_at = now(), cancelled_by = $2, cancel_reason = $3
        WHERE id = $1 AND status = 'draft'
        "#,
    )
    .bind(id)
    .bind(user.id)
    .bind(request.reason)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Conflict(
            "only draft stock adjustments can be cancelled".to_string(),
        ));
    }

    find_stock_adjustment(pool, id).await
}

pub async fn list_stock_conversions(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<StockConversionResponse>> {
    let search = query.search_pattern();
    let statuses = parse_status_filter(
        query.status.as_deref(),
        STOCK_CONVERSION_STATUSES,
        "stock conversion",
    )?;
    let sql = format!(
        "{STOCK_CONVERSION_SELECT}
        WHERE ($1 = '%%' OR lower(conversion_no) LIKE $1)
            AND ($2::date IS NULL OR converted_at::date >= $2)
            AND ($3::date IS NULL OR converted_at::date <= $3)
            AND (COALESCE(array_length($4::text[], 1), 0) = 0 OR status::text = ANY($4::text[]))
        ORDER BY id DESC
        LIMIT $5 OFFSET $6"
    );

    sqlx::query_as::<_, StockConversionResponse>(&sql)
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

pub async fn find_stock_conversion(
    pool: &PgPool,
    id: i64,
) -> AppResult<StockConversionDetailResponse> {
    let header_sql = format!("{STOCK_CONVERSION_SELECT} WHERE id = $1");
    let items_sql =
        format!("{STOCK_CONVERSION_ITEM_SELECT} WHERE stock_conversion_id = $1 ORDER BY id");

    let stock_conversion = sqlx::query_as::<_, StockConversionResponse>(&header_sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;
    let items = sqlx::query_as::<_, StockConversionItemResponse>(&items_sql)
        .bind(id)
        .fetch_all(pool)
        .await?;

    Ok(StockConversionDetailResponse {
        stock_conversion,
        items,
    })
}

pub async fn create_stock_conversion(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateStockConversionRequest,
) -> AppResult<StockConversionDetailResponse> {
    if request.items.is_empty() {
        return Err(AppError::Validation("items must not be empty".to_string()));
    }
    for item in &request.items {
        if item.from_product_sku_id == item.to_product_sku_id {
            return Err(AppError::Validation(
                "from_product_sku_id must be different from to_product_sku_id".to_string(),
            ));
        }
        validate_positive(item.from_quantity, "from_quantity")?;
        validate_positive(item.to_quantity, "to_quantity")?;
    }

    let mut tx = pool.begin().await?;
    let conversion_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO stock_conversions (converted_at, notes, created_by)
        VALUES (COALESCE($1, now()), $2, $3)
        RETURNING id
        "#,
    )
    .bind(request.converted_at)
    .bind(request.notes)
    .bind(user.id)
    .fetch_one(&mut *tx)
    .await?;

    for item in request.items {
        let from_product =
            load_product_sku_snapshot(&mut tx, item.from_product_sku_id, false).await?;
        let to_product = load_product_sku_snapshot(&mut tx, item.to_product_sku_id, false).await?;

        if from_product.product_id != to_product.product_id {
            return Err(AppError::Validation(
                "stock conversion SKUs must belong to the same product".to_string(),
            ));
        }
        if from_product.stock < item.from_quantity {
            return Err(AppError::Conflict(format!(
                "insufficient stock for product SKU {}",
                from_product.sku
            )));
        }

        sqlx::query(
            r#"
            INSERT INTO stock_conversion_items (
                stock_conversion_id,
                from_product_id, from_product_sku_id, from_product_name, from_product_sku,
                from_unit, from_stock_before, from_quantity, from_stock_after,
                to_product_id, to_product_sku_id, to_product_name, to_product_sku,
                to_unit, to_stock_before, to_quantity, to_stock_after, notes
            )
            VALUES (
                $1,
                $2, $3, $4, $5, $6, $7::numeric, $8::numeric, $9::numeric,
                $10, $11, $12, $13, $14, $15::numeric, $16::numeric, $17::numeric, $18
            )
            "#,
        )
        .bind(conversion_id)
        .bind(from_product.product_id)
        .bind(from_product.id)
        .bind(from_product.product_name)
        .bind(from_product.sku)
        .bind(from_product.unit)
        .bind(from_product.stock)
        .bind(item.from_quantity)
        .bind(from_product.stock - item.from_quantity)
        .bind(to_product.product_id)
        .bind(to_product.id)
        .bind(to_product.product_name)
        .bind(to_product.sku)
        .bind(to_product.unit)
        .bind(to_product.stock)
        .bind(item.to_quantity)
        .bind(to_product.stock + item.to_quantity)
        .bind(item.notes)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    find_stock_conversion(pool, conversion_id).await
}

pub async fn post_stock_conversion(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
) -> AppResult<StockConversionDetailResponse> {
    let mut tx = pool.begin().await?;
    let status: String =
        sqlx::query_scalar("SELECT status::text FROM stock_conversions WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(AppError::NotFound)?;
    if status != "draft" {
        return Err(AppError::Conflict(
            "stock conversion must be in draft status".to_string(),
        ));
    }

    let items = load_stock_conversion_items(&mut tx, id).await?;
    if items.is_empty() {
        return Err(AppError::Validation("items must not be empty".to_string()));
    }

    for item in items {
        let from_product =
            load_product_sku_snapshot(&mut tx, item.from_product_sku_id, true).await?;
        let to_product = load_product_sku_snapshot(&mut tx, item.to_product_sku_id, true).await?;

        if (from_product.stock - item.from_stock_before).abs() > f64::EPSILON {
            return Err(AppError::Conflict(format!(
                "product SKU {} stock changed after conversion was drafted",
                item.from_product_sku
            )));
        }
        if (to_product.stock - item.to_stock_before).abs() > f64::EPSILON {
            return Err(AppError::Conflict(format!(
                "product SKU {} stock changed after conversion was drafted",
                item.to_product_sku
            )));
        }
        if from_product.stock < item.from_quantity {
            return Err(AppError::Conflict(format!(
                "insufficient stock for product SKU {}",
                item.from_product_sku
            )));
        }

        sqlx::query("UPDATE product_skus SET stock = $2::numeric WHERE id = $1")
            .bind(item.from_product_sku_id)
            .bind(item.from_stock_after)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE product_skus SET stock = $2::numeric WHERE id = $1")
            .bind(item.to_product_sku_id)
            .bind(item.to_stock_after)
            .execute(&mut *tx)
            .await?;

        insert_stock_movement(
            &mut tx,
            item.from_product_id,
            item.from_product_sku_id,
            "conversion_out",
            item.from_quantity,
            item.from_stock_before,
            item.from_stock_after,
            item.notes.as_deref(),
            Some(user.id),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(id),
            Some(item.id),
        )
        .await?;
        insert_stock_movement(
            &mut tx,
            item.to_product_id,
            item.to_product_sku_id,
            "conversion_in",
            item.to_quantity,
            item.to_stock_before,
            item.to_stock_after,
            item.notes.as_deref(),
            Some(user.id),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(id),
            Some(item.id),
        )
        .await?;
    }

    sqlx::query(
        "UPDATE stock_conversions SET status = 'posted', posted_at = now(), posted_by = $2 WHERE id = $1",
    )
    .bind(id)
    .bind(user.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    find_stock_conversion(pool, id).await
}

pub async fn cancel_stock_conversion(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<StockConversionDetailResponse> {
    let result = sqlx::query(
        r#"
        UPDATE stock_conversions
        SET status = 'cancelled', cancelled_at = now(), cancelled_by = $2, cancel_reason = $3
        WHERE id = $1 AND status = 'draft'
        "#,
    )
    .bind(id)
    .bind(user.id)
    .bind(request.reason)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Conflict(
            "only draft stock conversions can be cancelled".to_string(),
        ));
    }

    find_stock_conversion(pool, id).await
}

pub async fn list_sales(
    pool: &PgPool,
    user: &AuthUser,
    query: ListQuery,
) -> AppResult<Vec<SaleResponse>> {
    let search = query.search_pattern();
    let statuses = parse_status_filter(query.status.as_deref(), SALE_STATUSES, "sale")?;
    let cashier_filter = if user.role == "admin" {
        None
    } else {
        Some(user.id)
    };
    let sql = format!(
        "{SALE_SELECT}
        WHERE ($1 = '%%' OR lower(s.transaction_no) LIKE $1 OR lower(u.name) LIKE $1)
            AND ($2::date IS NULL OR s.sold_at::date >= $2)
            AND ($3::date IS NULL OR s.sold_at::date <= $3)
            AND ($4::bigint IS NULL OR s.cashier_id = $4)
            AND (COALESCE(array_length($5::text[], 1), 0) = 0 OR s.status::text = ANY($5::text[]))
        ORDER BY s.id DESC
        LIMIT $6 OFFSET $7"
    );

    sqlx::query_as::<_, SaleResponse>(&sql)
        .bind(search)
        .bind(query.from)
        .bind(query.to)
        .bind(cashier_filter)
        .bind(statuses)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_sale(pool: &PgPool, user: &AuthUser, id: i64) -> AppResult<SaleDetailResponse> {
    let cashier_filter = if user.role == "admin" {
        None
    } else {
        Some(user.id)
    };
    let header_sql = format!(
        "{SALE_SELECT}
        WHERE s.id = $1 AND ($2::bigint IS NULL OR s.cashier_id = $2)"
    );
    let item_sql = format!("{SALE_ITEM_SELECT} WHERE sale_id = $1 ORDER BY id");

    let sale = sqlx::query_as::<_, SaleResponse>(&header_sql)
        .bind(id)
        .bind(cashier_filter)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;
    let items = sqlx::query_as::<_, SaleItemResponse>(&item_sql)
        .bind(id)
        .fetch_all(pool)
        .await?;

    Ok(SaleDetailResponse { sale, items })
}

pub async fn create_sale(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateSaleRequest,
) -> AppResult<SaleDetailResponse> {
    if request.items.is_empty() {
        return Err(AppError::Validation("items must not be empty".to_string()));
    }
    validate_non_negative(request.paid_amount, "paid_amount")?;

    let mut tx = pool.begin().await?;
    let mut subtotal = 0.0;
    let mut prepared_items = Vec::new();

    for item in request.items {
        validate_positive(item.quantity, "quantity")?;
        let product = load_product_sku_snapshot(&mut tx, item.product_sku_id, true).await?;
        if product.stock < item.quantity {
            return Err(AppError::Conflict(format!(
                "insufficient stock for product SKU {}",
                product.sku
            )));
        }
        subtotal += item.quantity * product.selling_price;
        prepared_items.push((item.quantity, product));
    }

    if request.paid_amount < subtotal {
        return Err(AppError::Validation(
            "paid_amount must be greater than or equal to total_amount".to_string(),
        ));
    }

    let sale_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO sales (
            sold_at, cashier_id, subtotal_amount, total_amount, paid_amount, change_amount, notes
        )
        VALUES (
            COALESCE($1, now()), $2, $3::numeric, $3::numeric, $4::numeric, ($4 - $3)::numeric, $5
        )
        RETURNING id
        "#,
    )
    .bind(request.sold_at)
    .bind(user.id)
    .bind(subtotal)
    .bind(request.paid_amount)
    .bind(request.notes)
    .fetch_one(&mut *tx)
    .await?;

    for (quantity, product) in prepared_items {
        let sale_item_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO sale_items (
                sale_id, product_id, product_sku_id, product_name, product_sku, unit, quantity, unit_price
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8::numeric)
            RETURNING id
            "#,
        )
        .bind(sale_id)
        .bind(product.product_id)
        .bind(product.id)
        .bind(product.product_name)
        .bind(product.sku)
        .bind(product.unit)
        .bind(quantity)
        .bind(product.selling_price)
        .fetch_one(&mut *tx)
        .await?;

        let stock_before = product.stock;
        let stock_after = stock_before - quantity;
        sqlx::query("UPDATE product_skus SET stock = $2::numeric WHERE id = $1")
            .bind(product.id)
            .bind(stock_after)
            .execute(&mut *tx)
            .await?;
        insert_stock_movement(
            &mut tx,
            product.product_id,
            product.id,
            "sale",
            quantity,
            stock_before,
            stock_after,
            Some("Sale completed"),
            Some(user.id),
            Some(sale_id),
            Some(sale_item_id),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;
    }

    tx.commit().await?;
    find_sale(pool, user, sale_id).await
}

pub async fn cancel_sale(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<SaleDetailResponse> {
    user.require_permission(pool, "pos").await?;
    let result = sqlx::query(
        r#"
        UPDATE sales
        SET status = 'cancelled', cancelled_at = now(), cancelled_by = $2, cancel_reason = $3
        WHERE id = $1 AND status = 'completed'
        "#,
    )
    .bind(id)
    .bind(user.id)
    .bind(request.reason)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Conflict("sale cannot be cancelled".to_string()));
    }

    find_sale(pool, user, id).await
}

pub async fn list_payroll_slips(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<PayrollSlipResponse>> {
    let search = query.search_pattern();
    let sql = format!(
        "{PAYROLL_SELECT}
        WHERE ($1 = '%%' OR lower(slip_no) LIKE $1 OR lower(employee_name) LIKE $1)
            AND ($2::date IS NULL OR period_start >= $2)
            AND ($3::date IS NULL OR period_end <= $3)
        ORDER BY id DESC
        LIMIT $4 OFFSET $5"
    );

    sqlx::query_as::<_, PayrollSlipResponse>(&sql)
        .bind(search)
        .bind(query.from)
        .bind(query.to)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn find_payroll_slip(pool: &PgPool, id: i64) -> AppResult<PayrollSlipResponse> {
    let sql = format!("{PAYROLL_SELECT} WHERE id = $1");

    sqlx::query_as::<_, PayrollSlipResponse>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create_payroll_slip(
    pool: &PgPool,
    user: &AuthUser,
    request: CreatePayrollSlipRequest,
) -> AppResult<PayrollSlipResponse> {
    if request.period_start > request.period_end {
        return Err(AppError::Validation(
            "period_start must be less than or equal to period_end".to_string(),
        ));
    }
    let bonus = request.bonus.unwrap_or(0.0);
    let deduction = request.deduction.unwrap_or(0.0);
    validate_non_negative(bonus, "bonus")?;
    validate_non_negative(deduction, "deduction")?;

    let employee = sqlx::query(
        r#"
        SELECT name, position, base_salary::float8 AS base_salary
        FROM employees
        WHERE id = $1 AND deleted_at IS NULL AND is_active = TRUE
        "#,
    )
    .bind(request.employee_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let base_salary: f64 = employee.get("base_salary");
    if base_salary + bonus - deduction < 0.0 {
        return Err(AppError::Validation(
            "net_salary must not be negative".to_string(),
        ));
    }

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO payroll_slips (
            employee_id, employee_name, employee_position, period_start, period_end,
            base_salary, bonus, deduction, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6::numeric, $7::numeric, $8::numeric, $9)
        RETURNING id
        "#,
    )
    .bind(request.employee_id)
    .bind(employee.get::<String, _>("name"))
    .bind(employee.get::<String, _>("position"))
    .bind(request.period_start)
    .bind(request.period_end)
    .bind(base_salary)
    .bind(bonus)
    .bind(deduction)
    .bind(user.id)
    .fetch_one(pool)
    .await?;

    find_payroll_slip(pool, id).await
}

#[derive(Debug, Clone)]
struct ProductSkuSnapshot {
    id: i64,
    product_id: i64,
    product_name: String,
    sku: String,
    unit: String,
    stock: f64,
    selling_price: f64,
}

async fn load_product_sku_snapshot(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    for_update: bool,
) -> AppResult<ProductSkuSnapshot> {
    let lock = if for_update { " FOR UPDATE" } else { "" };
    let sql = format!(
        r#"
        SELECT
            ps.id,
            ps.product_id,
            p.name AS product_name,
            ps.sku,
            u.code AS unit,
            ps.stock::float8 AS stock,
            ps.selling_price::float8 AS selling_price
        FROM product_skus ps
        JOIN products p ON p.id = ps.product_id
        JOIN units u ON u.id = ps.unit_id
        WHERE ps.id = $1
            AND ps.deleted_at IS NULL
            AND ps.is_active = TRUE
            AND p.deleted_at IS NULL
            AND p.is_active = TRUE
        {lock}
        "#
    );

    let row = sqlx::query(&sql)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(ProductSkuSnapshot {
        id: row.get("id"),
        product_id: row.get("product_id"),
        product_name: row.get("product_name"),
        sku: row.get("sku"),
        unit: row.get("unit"),
        stock: row.get("stock"),
        selling_price: row.get("selling_price"),
    })
}

async fn load_purchase_order_item(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    for_update: bool,
) -> AppResult<PurchaseOrderItemResponse> {
    let lock = if for_update { " FOR UPDATE" } else { "" };
    let sql = format!("{PURCHASE_ORDER_ITEM_SELECT} WHERE id = $1{lock}");

    sqlx::query_as::<_, PurchaseOrderItemResponse>(&sql)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(AppError::NotFound)
}

async fn load_goods_receipt_items(
    tx: &mut Transaction<'_, Postgres>,
    goods_receipt_id: i64,
) -> AppResult<Vec<GoodsReceiptItemResponse>> {
    let sql = format!("{GOODS_RECEIPT_ITEM_SELECT} WHERE goods_receipt_id = $1 ORDER BY id");

    sqlx::query_as::<_, GoodsReceiptItemResponse>(&sql)
        .bind(goods_receipt_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(Into::into)
}

async fn load_stock_adjustment_items(
    tx: &mut Transaction<'_, Postgres>,
    stock_adjustment_id: i64,
) -> AppResult<Vec<StockAdjustmentItemResponse>> {
    let sql = format!("{STOCK_ADJUSTMENT_ITEM_SELECT} WHERE stock_adjustment_id = $1 ORDER BY id");

    sqlx::query_as::<_, StockAdjustmentItemResponse>(&sql)
        .bind(stock_adjustment_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(Into::into)
}

async fn load_stock_conversion_items(
    tx: &mut Transaction<'_, Postgres>,
    stock_conversion_id: i64,
) -> AppResult<Vec<StockConversionItemResponse>> {
    let sql = format!("{STOCK_CONVERSION_ITEM_SELECT} WHERE stock_conversion_id = $1 ORDER BY id");

    sqlx::query_as::<_, StockConversionItemResponse>(&sql)
        .bind(stock_conversion_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(Into::into)
}

async fn update_purchase_order_status(
    tx: &mut Transaction<'_, Postgres>,
    purchase_order_id: i64,
) -> AppResult<()> {
    let status: String = sqlx::query_scalar(
        r#"
        SELECT CASE
            WHEN bool_and(quantity_received >= quantity_ordered) THEN 'received'
            WHEN bool_or(quantity_received > 0) THEN 'partially_received'
            ELSE 'ordered'
        END
        FROM purchase_order_items
        WHERE purchase_order_id = $1
        "#,
    )
    .bind(purchase_order_id)
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query("UPDATE purchase_orders SET status = $2::purchase_order_status WHERE id = $1")
        .bind(purchase_order_id)
        .bind(status)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn insert_stock_movement(
    tx: &mut Transaction<'_, Postgres>,
    product_id: i64,
    product_sku_id: i64,
    movement_type: &str,
    quantity: f64,
    stock_before: f64,
    stock_after: f64,
    notes: Option<&str>,
    created_by: Option<i64>,
    sale_id: Option<i64>,
    sale_item_id: Option<i64>,
    goods_receipt_id: Option<i64>,
    goods_receipt_item_id: Option<i64>,
    stock_adjustment_id: Option<i64>,
    stock_adjustment_item_id: Option<i64>,
    stock_conversion_id: Option<i64>,
    stock_conversion_item_id: Option<i64>,
) -> AppResult<()> {
    validate_positive(quantity, "quantity")?;
    sqlx::query(
        r#"
        INSERT INTO stock_movements (
            product_id, product_sku_id, sale_id, sale_item_id, goods_receipt_id,
            goods_receipt_item_id, stock_adjustment_id, stock_adjustment_item_id,
            stock_conversion_id, stock_conversion_item_id, movement_type, quantity,
            stock_before, stock_after, notes, created_by
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11::stock_movement_type,
            $12::numeric, $13::numeric, $14::numeric, $15, $16
        )
        "#,
    )
    .bind(product_id)
    .bind(product_sku_id)
    .bind(sale_id)
    .bind(sale_item_id)
    .bind(goods_receipt_id)
    .bind(goods_receipt_item_id)
    .bind(stock_adjustment_id)
    .bind(stock_adjustment_item_id)
    .bind(stock_conversion_id)
    .bind(stock_conversion_item_id)
    .bind(movement_type)
    .bind(quantity)
    .bind(stock_before)
    .bind(stock_after)
    .bind(notes)
    .bind(created_by)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
