use serde::{Deserialize, Serialize};

/// Filter katalog publik. Beda dari `ListQuery`: ada `category_id`,
/// tanpa filter tanggal/status internal.
#[derive(Debug, Deserialize)]
pub struct PublicCatalogQuery {
    pub search: Option<String>,
    pub category_id: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl PublicCatalogQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(50).clamp(1, 100)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }

    pub fn search_pattern(&self) -> String {
        format!("%{}%", self.search.as_deref().unwrap_or("").to_lowercase())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicProductItem {
    pub product_sku_id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
    pub selling_price: f64,
    pub stock: f64,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub photo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicCategory {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreInfoResponse {
    pub store_name: String,
    pub operating_hours: String,
    pub bank_accounts: String,
    pub ewallet_numbers: String,
    pub qris_image_url: Option<String>,
    pub delivery_fee: f64,
    pub shop_whatsapp: String,
    pub payment_mode: String,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutItemRequest {
    pub product_sku_id: i64,
    pub quantity: f64,
}

/// Body checkout publik. Sengaja TIDAK ada field `member_id`: atribusi member
/// selalu diambil dari JWT member (`OptionalMember`), tidak pernah dari body
/// (anti-spoof). Checkout member memakai nama/telepon profil; guest memakai body.
#[derive(Debug, Deserialize)]
pub struct CheckoutRequest {
    pub customer_name: String,
    pub customer_phone: String,
    pub fulfillment_method: String,
    pub delivery_address: Option<String>,
    pub payment_method: String,
    pub notes: Option<String>,
    pub items: Vec<CheckoutItemRequest>,
}

#[derive(Debug, Serialize)]
pub struct CheckoutItemResponse {
    pub product_sku: String,
    pub product_name: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub line_total: f64,
}

#[derive(Debug, Serialize)]
pub struct CheckoutPaymentInfo {
    pub bank_accounts: String,
    pub ewallet_numbers: String,
    pub qris_image_url: Option<String>,
    pub total_amount: f64,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub order_id: i64,
    pub order_no: String,
    pub fulfillment_method: String,
    pub payment_method: String,
    pub total_amount: f64,
    pub items: Vec<CheckoutItemResponse>,
    pub payment_info: Option<CheckoutPaymentInfo>,
    /// Snap pembayaran online (`payment_method = midtrans`): token + URL
    /// bayar + `order_id` Midtrans (baris `midtrans_payments`, bukan
    /// `online_orders`). Bernilai `None` untuk `pay_at_store`/`transfer`.
    pub snap_token: Option<String>,
    pub redirect_url: Option<String>,
    pub midtrans_order_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OnlineOrderItemResponse {
    pub id: i64,
    pub product_sku_id: i64,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub line_total: f64,
}

#[derive(Debug, Serialize)]
pub struct OnlineOrderResponse {
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
    pub member_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct OnlineOrderDetailResponse {
    pub order: OnlineOrderResponse,
    pub items: Vec<OnlineOrderItemResponse>,
}

/// Pembaruan parsial pengaturan toko oleh kasir/admin.
/// `None` = tidak diubah; `Some` = ganti nilai (string kosong = kosongkan).
#[derive(Debug, Deserialize)]
pub struct UpdateStoreSettingsRequest {
    pub store_name: Option<String>,
    pub operating_hours: Option<String>,
    pub bank_accounts: Option<String>,
    pub ewallet_numbers: Option<String>,
    pub delivery_fee: Option<f64>,
    pub shop_whatsapp: Option<String>,
    pub payment_mode: Option<String>,
}
