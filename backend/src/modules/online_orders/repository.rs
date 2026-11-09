use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    error::{AppError, AppResult},
    models::{ListQuery, PaginationMeta},
};

use super::{
    dto::{PublicCatalogQuery, PublicCategory, PublicProductItem},
    model::{
        CatalogRow, CheckoutSkuRow, LockedSkuRow, NewOrderRow, OrderItemRow, OrderRow,
        PublicCategoryRow,
    },
};

const CATALOG_WHERE: &str = r#"
    WHERE ps.deleted_at IS NULL
        AND p.deleted_at IS NULL
        AND u.deleted_at IS NULL
        AND ps.is_active
        AND p.is_active
        AND ($1 = '%%' OR lower(p.name) LIKE $1 OR lower(ps.sku) LIKE $1)
        AND ($4::bigint IS NULL OR p.category_id = $4)
"#;

const CATALOG_SELECT: &str = r#"
    SELECT
        ps.id AS product_sku_id,
        ps.product_id,
        p.name AS product_name,
        ps.sku AS product_sku,
        u.code AS unit,
        ps.selling_price::float8 AS selling_price,
        ps.stock::float8 AS stock,
        p.category_id,
        c.name AS category_name,
        p.image_url AS image_key,
        ps.created_at
    FROM product_skus ps
    JOIN products p ON p.id = ps.product_id
    JOIN units u ON u.id = ps.unit_id
    LEFT JOIN product_categories c ON c.id = p.category_id
"#;

pub async fn list_public_categories(pool: &PgPool) -> AppResult<Vec<PublicCategory>> {
    let rows = sqlx::query_as::<_, PublicCategoryRow>(
        r#"
        SELECT id, name
        FROM product_categories
        WHERE deleted_at IS NULL AND is_active
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| PublicCategory {
            id: row.id,
            name: row.name,
        })
        .collect())
}

pub async fn list_catalog(
    pool: &PgPool,
    query: &PublicCatalogQuery,
    public_url_base: Option<&str>,
) -> AppResult<(Vec<PublicProductItem>, PaginationMeta)> {
    let search = query.search_pattern();
    let limit = query.limit();
    let offset = query.offset();

    let sql =
        format!("{CATALOG_SELECT} {CATALOG_WHERE} ORDER BY p.name, ps.sku LIMIT $2 OFFSET $3");
    let rows = sqlx::query_as::<_, CatalogRow>(&sql)
        .bind(&search)
        .bind(limit)
        .bind(offset)
        .bind(query.category_id)
        .fetch_all(pool)
        .await?;

    let count_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM product_skus ps
        JOIN products p ON p.id = ps.product_id
        JOIN units u ON u.id = ps.unit_id
        LEFT JOIN product_categories c ON c.id = p.category_id
        {CATALOG_WHERE}
        "#
    );
    let total: i64 = sqlx::query_scalar(&count_sql)
        .bind(&search)
        .bind(limit)
        .bind(offset)
        .bind(query.category_id)
        .fetch_one(pool)
        .await?;

    let items = rows
        .into_iter()
        .map(|row| PublicProductItem {
            product_sku_id: row.product_sku_id,
            product_id: row.product_id,
            product_name: row.product_name,
            product_sku: row.product_sku,
            unit: row.unit,
            selling_price: row.selling_price,
            stock: row.stock,
            category_id: row.category_id,
            category_name: row.category_name,
            photo_url: row.image_key.as_deref().and_then(|key| {
                public_url_base.map(|base| format!("{}/{}", base.trim_end_matches('/'), key))
            }),
        })
        .collect();

    Ok((items, PaginationMeta::new(limit, offset, total)))
}

/// Muat SKU untuk item checkout: hanya yang aktif dan belum dihapus.
/// Stok dicek ulang di service; pemotongan terjadi saat packing (Fase 3).
pub async fn load_checkout_skus(pool: &PgPool, sku_ids: &[i64]) -> AppResult<Vec<CheckoutSkuRow>> {
    let rows = sqlx::query_as::<_, CheckoutSkuRow>(
        r#"
        SELECT
            ps.id AS product_sku_id,
            ps.product_id,
            p.name AS product_name,
            ps.sku AS product_sku,
            u.code AS unit,
            ps.selling_price::float8 AS selling_price,
            ps.stock::float8 AS stock,
            (ps.is_active AND p.is_active) AS is_active
        FROM product_skus ps
        JOIN products p ON p.id = ps.product_id
        JOIN units u ON u.id = ps.unit_id
        WHERE ps.id = ANY($1)
            AND ps.deleted_at IS NULL
            AND p.deleted_at IS NULL
        "#,
    )
    .bind(sku_ids)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub struct NewOrderItem {
    pub product_id: i64,
    pub product_sku_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
    pub quantity: f64,
    pub unit_price: f64,
}

pub struct NewOrderData {
    pub customer_name: String,
    pub customer_phone: String,
    pub fulfillment_method: String,
    pub delivery_address: Option<String>,
    pub payment_method: String,
    pub notes: Option<String>,
    pub subtotal: f64,
    pub delivery_fee: f64,
    pub items: Vec<NewOrderItem>,
}

pub async fn insert_order(pool: &PgPool, order: NewOrderData) -> AppResult<NewOrderRow> {
    let mut tx = pool.begin().await?;

    let saved = sqlx::query_as::<_, NewOrderRow>(
        r#"
        INSERT INTO online_orders (
            customer_name, customer_phone, fulfillment_method, delivery_address,
            payment_method, notes, subtotal_amount, delivery_fee, total_amount
        )
        VALUES (
            $1, $2, $3::online_order_fulfillment_method, $4,
            $5::online_order_payment_method, $6, $7, $8, $7 + $8
        )
        RETURNING id, order_no
        "#,
    )
    .bind(&order.customer_name)
    .bind(&order.customer_phone)
    .bind(&order.fulfillment_method)
    .bind(&order.delivery_address)
    .bind(&order.payment_method)
    .bind(&order.notes)
    .bind(order.subtotal)
    .bind(order.delivery_fee)
    .fetch_one(&mut *tx)
    .await?;

    for item in &order.items {
        sqlx::query(
            r#"
            INSERT INTO online_order_items (
                online_order_id, product_id, product_sku_id,
                product_name, product_sku, unit, quantity, unit_price
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(saved.id)
        .bind(item.product_id)
        .bind(item.product_sku_id)
        .bind(&item.product_name)
        .bind(&item.product_sku)
        .bind(&item.unit)
        .bind(item.quantity)
        .bind(item.unit_price)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(saved)
}

pub async fn get_setting(pool: &PgPool, key: &str) -> AppResult<Option<String>> {
    let value: Option<String> =
        sqlx::query_scalar("SELECT value FROM store_settings WHERE key = $1")
            .bind(key)
            .fetch_optional(pool)
            .await?;

    Ok(value)
}

pub async fn set_setting(
    pool: &PgPool,
    key: &str,
    value: &str,
    updated_by: Option<i64>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO store_settings (key, value, updated_by)
        VALUES ($1, $2, $3)
        ON CONFLICT (key) DO UPDATE SET
            value = EXCLUDED.value,
            updated_by = EXCLUDED.updated_by,
            updated_at = now()
        "#,
    )
    .bind(key)
    .bind(value)
    .bind(updated_by)
    .execute(pool)
    .await?;

    Ok(())
}

pub const ORDER_STATUSES: &[&str] = &[
    "new",
    "packing",
    "ready",
    "delivering",
    "completed",
    "cancelled",
];

const ORDER_SELECT: &str = r#"
    SELECT
        id,
        order_no,
        customer_name,
        customer_phone,
        fulfillment_method::text AS fulfillment_method,
        delivery_address,
        payment_method::text AS payment_method,
        payment_status::text AS payment_status,
        status::text AS status,
        subtotal_amount::float8 AS subtotal_amount,
        delivery_fee::float8 AS delivery_fee,
        total_amount::float8 AS total_amount,
        notes,
        sale_id,
        cancel_reason,
        created_at
    FROM online_orders
"#;

/// Daftar pesanan untuk dashboard kasir: filter status (koma), cari, paginasi.
pub async fn list_orders(
    pool: &PgPool,
    query: &ListQuery,
) -> AppResult<(Vec<OrderRow>, PaginationMeta)> {
    let search = query.search_pattern();
    let limit = query.limit();
    let offset = query.offset();
    let statuses = parse_order_statuses(query.status.as_deref())?;

    let sql = format!(
        r#"{ORDER_SELECT}
        WHERE ($1 = '%%'
            OR lower(order_no) LIKE $1
            OR lower(customer_name) LIKE $1
            OR lower(customer_phone) LIKE $1)
            AND (COALESCE(array_length($4::text[], 1), 0) = 0 OR status::text = ANY($4::text[]))
        ORDER BY id DESC
        LIMIT $2 OFFSET $3"#
    );
    let rows = sqlx::query_as::<_, OrderRow>(&sql)
        .bind(&search)
        .bind(limit)
        .bind(offset)
        .bind(&statuses)
        .fetch_all(pool)
        .await?;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM online_orders
        WHERE ($1 = '%%'
            OR lower(order_no) LIKE $1
            OR lower(customer_name) LIKE $1
            OR lower(customer_phone) LIKE $1)
            AND (COALESCE(array_length($2::text[], 1), 0) = 0 OR status::text = ANY($2::text[]))
        "#,
    )
    .bind(&search)
    .bind(&statuses)
    .fetch_one(pool)
    .await?;

    Ok((rows, PaginationMeta::new(limit, offset, total)))
}

fn parse_order_statuses(status: Option<&str>) -> AppResult<Vec<String>> {
    let Some(status) = status else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for value in status.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if !ORDER_STATUSES.contains(&value) {
            return Err(AppError::Validation(format!(
                "unknown online order status: {value}"
            )));
        }
        out.push(value.to_string());
    }
    Ok(out)
}

pub async fn find_order(pool: &PgPool, id: i64) -> AppResult<Option<OrderRow>> {
    let row = sqlx::query_as::<_, OrderRow>(&format!("{ORDER_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(row)
}

pub async fn order_items(pool: &PgPool, order_id: i64) -> AppResult<Vec<OrderItemRow>> {
    let rows = sqlx::query_as::<_, OrderItemRow>(
        r#"
        SELECT
            id,
            product_sku_id,
            product_name,
            product_sku,
            unit,
            quantity::float8 AS quantity,
            unit_price::float8 AS unit_price,
            line_total::float8 AS line_total
        FROM online_order_items
        WHERE online_order_id = $1
        ORDER BY id
        "#,
    )
    .bind(order_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Kunci header pesanan dalam transaksi transisi status.
pub async fn lock_order(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
) -> AppResult<Option<OrderRow>> {
    let row = sqlx::query_as::<_, OrderRow>(&format!("{ORDER_SELECT} WHERE id = $1 FOR UPDATE"))
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?;

    Ok(row)
}

/// Kunci baris SKU untuk pemotongan/pengembalian stok.
pub async fn lock_sku(
    tx: &mut Transaction<'_, Postgres>,
    product_sku_id: i64,
) -> AppResult<Option<LockedSkuRow>> {
    let row = sqlx::query_as::<_, LockedSkuRow>(
        r#"
        SELECT product_id, id AS product_sku_id, stock::float8 AS stock
        FROM product_skus
        WHERE id = $1 AND deleted_at IS NULL
        FOR UPDATE
        "#,
    )
    .bind(product_sku_id)
    .fetch_optional(&mut **tx)
    .await?;

    Ok(row)
}

async fn write_stock(
    tx: &mut Transaction<'_, Postgres>,
    product_sku_id: i64,
    stock_after: f64,
) -> AppResult<()> {
    sqlx::query("UPDATE product_skus SET stock = $2::numeric WHERE id = $1")
        .bind(product_sku_id)
        .bind(stock_after)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn insert_movement(
    tx: &mut Transaction<'_, Postgres>,
    product_id: i64,
    product_sku_id: i64,
    movement_type: &str,
    quantity: f64,
    stock_before: f64,
    stock_after: f64,
    notes: &str,
    created_by: i64,
    sale_id: Option<i64>,
    sale_item_id: Option<i64>,
    online_order_id: Option<i64>,
    online_order_item_id: Option<i64>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO stock_movements (
            product_id, product_sku_id, movement_type, quantity,
            stock_before, stock_after, notes, created_by,
            sale_id, sale_item_id, online_order_id, online_order_item_id
        )
        VALUES (
            $1, $2, $3::stock_movement_type, $4::numeric,
            $5::numeric, $6::numeric, $7, $8,
            $9, $10, $11, $12
        )
        "#,
    )
    .bind(product_id)
    .bind(product_sku_id)
    .bind(movement_type)
    .bind(quantity)
    .bind(stock_before)
    .bind(stock_after)
    .bind(notes)
    .bind(created_by)
    .bind(sale_id)
    .bind(sale_item_id)
    .bind(online_order_id)
    .bind(online_order_item_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Potong stok satu item saat packing + catat movement tertaut pesanan.
pub async fn cut_stock_for_pack(
    tx: &mut Transaction<'_, Postgres>,
    item: &OrderItemRow,
    created_by: i64,
    order_id: i64,
) -> AppResult<()> {
    let sku = lock_sku(tx, item.product_sku_id).await?.ok_or_else(|| {
        AppError::Conflict(format!("{} is no longer available", item.product_sku))
    })?;
    if sku.stock < item.quantity {
        return Err(AppError::Conflict(format!(
            "{} only has {} {} in stock",
            item.product_name, sku.stock, item.unit
        )));
    }

    let stock_after = sku.stock - item.quantity;
    write_stock(tx, item.product_sku_id, stock_after).await?;
    insert_movement(
        tx,
        sku.product_id,
        item.product_sku_id,
        "sale",
        item.quantity,
        sku.stock,
        stock_after,
        "Online order packed",
        created_by,
        None,
        None,
        Some(order_id),
        Some(item.id),
    )
    .await?;

    Ok(())
}

/// Kembalikan stok satu item saat batal dari packing + movement koreksi.
pub async fn restore_stock_for_cancel(
    tx: &mut Transaction<'_, Postgres>,
    item: &OrderItemRow,
    created_by: i64,
    order_id: i64,
) -> AppResult<()> {
    let sku = lock_sku(tx, item.product_sku_id).await?.ok_or_else(|| {
        AppError::Conflict(format!("{} is no longer available", item.product_sku))
    })?;

    let stock_after = sku.stock + item.quantity;
    write_stock(tx, item.product_sku_id, stock_after).await?;
    insert_movement(
        tx,
        sku.product_id,
        item.product_sku_id,
        "correction",
        item.quantity,
        sku.stock,
        stock_after,
        "Online order cancelled, stock restored",
        created_by,
        None,
        None,
        Some(order_id),
        Some(item.id),
    )
    .await?;

    Ok(())
}

/// Tautkan movement packing ke penjualan yang dibuat saat complete.
pub async fn link_movements_to_sale(
    tx: &mut Transaction<'_, Postgres>,
    order_item_id: i64,
    sale_id: i64,
    sale_item_id: i64,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE stock_movements
        SET sale_id = $2, sale_item_id = $3
        WHERE online_order_item_id = $1 AND movement_type = 'sale'
        "#,
    )
    .bind(order_item_id)
    .bind(sale_id)
    .bind(sale_item_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn set_order_status(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    status: &str,
    processed_by: i64,
) -> AppResult<()> {
    let affected = sqlx::query(
        r#"
        UPDATE online_orders
        SET status = $2::online_order_status, processed_by = $3, updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(processed_by)
    .execute(&mut **tx)
    .await?;

    if affected.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

pub async fn set_order_cancelled(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    reason: Option<&str>,
    processed_by: i64,
) -> AppResult<()> {
    let affected = sqlx::query(
        r#"
        UPDATE online_orders
        SET status = 'cancelled', cancel_reason = $2, processed_by = $3, updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(reason)
    .bind(processed_by)
    .execute(&mut **tx)
    .await?;

    if affected.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

pub async fn set_order_paid(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    processed_by: i64,
) -> AppResult<()> {
    let affected = sqlx::query(
        r#"
        UPDATE online_orders
        SET payment_status = 'paid', processed_by = $2, updated_at = now()
        WHERE id = $1 AND payment_status = 'unpaid'
            AND status NOT IN ('completed', 'cancelled')
        "#,
    )
    .bind(id)
    .bind(processed_by)
    .execute(&mut **tx)
    .await?;

    if affected.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

/// Sesi register terbuka milik kasir (keputusan A: complete wajib sesi sendiri).
pub async fn find_cashier_open_session(
    tx: &mut Transaction<'_, Postgres>,
    cashier_id: i64,
) -> AppResult<Option<i64>> {
    let session = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT crs.id
        FROM cash_register_sessions crs
        JOIN cash_registers cr ON cr.id = crs.cash_register_id
        WHERE crs.opened_by = $1
            AND crs.status = 'open'
            AND cr.status = 'active'
            AND cr.deleted_at IS NULL
        ORDER BY crs.id DESC
        LIMIT 1
        "#,
    )
    .bind(cashier_id)
    .fetch_optional(&mut **tx)
    .await?;

    Ok(session)
}

/// Buat penjualan dari pesanan online TANPA memotong stok
/// (stok sudah dipotong saat packing). Mirror create_sale minus stok.
pub async fn insert_online_sale(
    tx: &mut Transaction<'_, Postgres>,
    order: &OrderRow,
    items: &[OrderItemRow],
    cashier_id: i64,
    session_id: i64,
) -> AppResult<(i64, Vec<(i64, i64)>)> {
    let sale_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO sales (
            cashier_id, cash_register_session_id, subtotal_amount, total_amount,
            paid_amount, change_amount, channel, online_order_id, notes
        )
        VALUES ($1, $2, $3::numeric, $4::numeric, $4::numeric, 0, 'online', $5, $6)
        RETURNING id
        "#,
    )
    .bind(cashier_id)
    .bind(session_id)
    .bind(order.subtotal_amount)
    .bind(order.total_amount)
    .bind(order.id)
    .bind(format!("Online order {}", order.order_no))
    .fetch_one(&mut **tx)
    .await?;

    let mut links = Vec::with_capacity(items.len());
    for item in items {
        let sale_item_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO sale_items (
                sale_id, product_id, product_sku_id,
                product_name, product_sku, unit, quantity, unit_price, unit_cost
            )
            SELECT $1, oi.product_id, oi.product_sku_id,
                oi.product_name, oi.product_sku, oi.unit, oi.quantity, oi.unit_price,
                ps.purchase_price
            FROM online_order_items oi
            JOIN product_skus ps ON ps.id = oi.product_sku_id
            WHERE oi.id = $2
            RETURNING id
            "#,
        )
        .bind(sale_id)
        .bind(item.id)
        .fetch_one(&mut **tx)
        .await?;
        links.push((item.id, sale_item_id));
    }

    Ok((sale_id, links))
}

pub async fn finalize_completed_order(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    sale_id: i64,
    processed_by: i64,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE online_orders
        SET status = 'completed', sale_id = $2, payment_status = 'paid',
            processed_by = $3, updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(sale_id)
    .bind(processed_by)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
