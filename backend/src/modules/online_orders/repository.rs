use sqlx::PgPool;

use crate::{error::AppResult, models::PaginationMeta};

use super::{
    dto::{PublicCatalogQuery, PublicCategory, PublicProductItem},
    model::{CatalogRow, CheckoutSkuRow, NewOrderRow, PublicCategoryRow},
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

    let sql = format!(
        "{CATALOG_SELECT} {CATALOG_WHERE} ORDER BY p.name, ps.sku LIMIT $2 OFFSET $3"
    );
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
pub async fn load_checkout_skus(
    pool: &PgPool,
    sku_ids: &[i64],
) -> AppResult<Vec<CheckoutSkuRow>> {
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
