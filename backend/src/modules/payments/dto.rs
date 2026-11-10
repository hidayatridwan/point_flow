use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::model::ChargeItem;

/// Permintaan charge Midtrans dari kasir POS (`source_type = pos`) atau
/// storefront online (`source_type = online` + `online_order_id`).
/// Tanpa `paid_amount`: pembayaran pending, stok belum berkurang.
#[derive(Debug, Deserialize)]
pub struct CreateMidtransChargeRequest {
    pub cash_register_session_id: i64,
    pub member_id: Option<i64>,
    pub notes: Option<String>,
    pub items: Vec<ChargeItem>,
    pub source_type: Option<String>,
    pub online_order_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct MidtransChargeResponse {
    pub order_id: String,
    pub snap_token: Option<String>,
    pub redirect_url: Option<String>,
    pub gross_amount: i64,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct MidtransPaymentResponse {
    pub order_id: String,
    pub source_type: String,
    pub gross_amount: i64,
    pub transaction_status: String,
    pub fraud_status: Option<String>,
    pub payment_type: Option<String>,
    pub snap_token: Option<String>,
    pub redirect_url: Option<String>,
    pub sale_id: Option<i64>,
    pub online_order_id: Option<i64>,
    pub expires_at: Option<DateTime<Utc>>,
    pub settled_at: Option<DateTime<Utc>>,
}

/// Payload notifikasi Midtrans. `status_code` dan `gross_amount` WAJIB
/// string mentah (dipakai verbatim untuk verifikasi `signature_key`).
#[derive(Debug, Serialize, Deserialize)]
pub struct MidtransNotificationPayload {
    pub order_id: String,
    pub status_code: String,
    pub gross_amount: String,
    pub signature_key: String,
    pub transaction_status: String,
    pub fraud_status: Option<String>,
    pub payment_type: Option<String>,
    #[serde(default)]
    pub va_numbers: Option<Vec<VaNumber>>,
    #[serde(default)]
    pub permata_va_number: Option<String>,
    #[serde(default)]
    pub bill_key: Option<String>,
    #[serde(default)]
    pub biller_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaNumber {
    pub bank: Option<String>,
    pub va_number: Option<String>,
}

/// Query polling kasir: `?refresh=true` menantang Midtrans (rekonsiliasi)
/// sebelum mengembalikan status tersimpan.
#[derive(Debug, Deserialize)]
pub struct GetPaymentQuery {
    pub refresh: Option<bool>,
}
