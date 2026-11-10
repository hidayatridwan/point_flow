use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

/// Baris `midtrans_payments`: satu transaksi Snap pending hingga terminal.
/// Stok/sale hanya berubah saat transisi pertama ke settlement (fase webhook).
#[derive(Debug, Clone, FromRow)]
pub struct MidtransPayment {
    pub id: i64,
    pub order_id: String,
    pub source_type: String,
    pub gross_amount: i32,
    pub snap_token: Option<String>,
    pub redirect_url: Option<String>,
    pub transaction_status: String,
    pub fraud_status: Option<String>,
    pub payment_type: Option<String>,
    pub va_info: Option<Value>,
    pub raw_notification: Option<Value>,
    pub sale_id: Option<i64>,
    pub online_order_id: Option<i64>,
    pub items: Option<Value>,
    pub member_id: Option<i64>,
    pub notes: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub settled_at: Option<DateTime<Utc>>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Item charge dari kasir/storefront: kuantitas fraksional boleh masuk,
/// nominal per-SKU dibulatkan ke rupiah saat kirim ke Midtrans.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargeItem {
    pub product_sku_id: i64,
    pub quantity: f64,
}
