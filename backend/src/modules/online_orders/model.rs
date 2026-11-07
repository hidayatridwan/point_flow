use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

/// Satu baris katalog publik: SKU aktif dengan harga jual dan stok saat ini.
/// Tidak memuat harga beli, supplier, atau data internal lain.
#[derive(Debug, FromRow, Serialize)]
pub struct CatalogRow {
    pub product_sku_id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
    pub selling_price: f64,
    pub stock: f64,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub image_key: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Kategori untuk filter katalog publik.
#[derive(Debug, FromRow, Serialize)]
pub struct PublicCategoryRow {
    pub id: i64,
    pub name: String,
}

/// SKU yang dibeli saat checkout: keaktifan, harga, dan stok dicek ulang di sini.
#[derive(Debug, FromRow)]
pub struct CheckoutSkuRow {
    pub product_sku_id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
    pub selling_price: f64,
    pub stock: f64,
    pub is_active: bool,
}

/// Pesanan yang baru dibuat (nomor dihasilkan database).
#[derive(Debug, FromRow)]
pub struct NewOrderRow {
    pub id: i64,
    pub order_no: String,
}

/// Header pesanan untuk dashboard kasir.
#[derive(Debug, Clone, FromRow)]
pub struct OrderRow {
    pub id: i64,
    pub order_no: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub fulfillment_method: String,
    pub delivery_address: Option<String>,
    pub payment_method: String,
    pub payment_status: String,
    pub status: String,
    pub subtotal_amount: f64,
    pub delivery_fee: f64,
    pub total_amount: f64,
    pub notes: Option<String>,
    pub sale_id: Option<i64>,
    pub cancel_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Item pesanan untuk dashboard kasir.
#[derive(Debug, FromRow)]
pub struct OrderItemRow {
    pub id: i64,
    pub product_sku_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub line_total: f64,
}

/// Snapshot SKU dengan kunci baris untuk pemotongan stok saat packing.
#[derive(Debug, FromRow)]
pub struct LockedSkuRow {
    pub product_id: i64,
    pub product_sku_id: i64,
    pub stock: f64,
}
