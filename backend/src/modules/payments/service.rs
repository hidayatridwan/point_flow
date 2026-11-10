use chrono::Utc;
use serde_json::{Value, json};
use sha2::{Digest, Sha512};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    config::MidtransSettings,
    error::{AppError, AppResult},
    models::SaleItemInput,
    services::{audit, cash_registers, documents},
};

use super::{
    dto::{
        CreateMidtransChargeRequest, MidtransChargeResponse, MidtransNotificationPayload,
        MidtransPaymentResponse,
    },
    model::MidtransPayment,
    repository,
};

/// Metode pembayaran Snap yang dibuka untuk Point Flow (keputusan terkunci).
pub const ENABLED_PAYMENTS: &[&str] =
    &["qris", "bank_transfer", "gopay", "shopeepay", "dana", "ovo"];

pub const SOURCE_POS: &str = "pos";
pub const SOURCE_ONLINE: &str = "online";

/// Mode toko yang mengizinkan charge Midtrans; selain itu kasir memakai
/// pencatatan transfer manual (nilai sama dengan mode di online_orders).
const PAYMENT_MODE_MIDTRANS: &str = "midtrans";

/// Status yang menutup pembayaran tanpa aksi lanjutan (`capture + challenge`
/// bukan terminal: masih menunggu keputusan review di dashboard Midtrans).
fn is_terminal_status(transaction_status: &str, fraud_status: Option<&str>) -> bool {
    match transaction_status {
        "settlement" | "expire" | "cancel" | "deny" | "failure" => true,
        "capture" => fraud_status != Some("challenge"),
        _ => false,
    }
}

/// Lunas menurut Midtrans: `settlement`, atau `capture` kartu kredit yang
/// lolos verifikasi fraud (`accept`).
pub fn is_settlement_status(transaction_status: &str, fraud_status: Option<&str>) -> bool {
    transaction_status == "settlement"
        || (transaction_status == "capture" && fraud_status == Some("accept"))
}

fn status_rank(transaction_status: &str) -> u8 {
    match transaction_status {
        "pending" => 0,
        "authorize" => 1,
        "capture" => 2,
        "settlement" => 3,
        "deny" | "cancel" | "expire" | "failure" => 4,
        _ => 0,
    }
}

/// Keputusan pemrosesan satu notifikasi/status terhadap baris tersimpan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationAction {
    /// Abaikan: duplikat, out-of-order, terminal, atau info pasca-settlement.
    Ignore,
    /// Catat status baru tanpa menyentuh stok/sale/order.
    UpdateStatus,
    /// Transisi lunas pertama: buat sale (POS) / tandai order lunas (online).
    Settle,
}

pub fn decide_notification_action(
    stored_status: &str,
    stored_fraud: Option<&str>,
    settled: bool,
    incoming_status: &str,
    incoming_fraud: Option<&str>,
) -> NotificationAction {
    // Guard idempotensi utama: settlement hanya dieksekusi sekali.
    if settled {
        return NotificationAction::Ignore;
    }
    // Info pasca-settlement (refund/partial_refund) tidak mengubah apa pun.
    if matches!(incoming_status, "refund" | "partial_refund") {
        return NotificationAction::Ignore;
    }
    if is_terminal_status(stored_status, stored_fraud) {
        return NotificationAction::Ignore;
    }
    // Notifikasi out-of-order: status tersimpan sudah lebih maju.
    if status_rank(incoming_status) < status_rank(stored_status) {
        return NotificationAction::Ignore;
    }
    if is_settlement_status(incoming_status, incoming_fraud) {
        return NotificationAction::Settle;
    }
    NotificationAction::UpdateStatus
}

/// Verifikasi `signature_key` Midtrans:
/// `SHA512(order_id + status_code + gross_amount + server_key)`.
/// `gross_amount`/`status_code` dipakai verbatim (string mentah notifikasi).
pub fn verify_signature(
    order_id: &str,
    status_code: &str,
    gross_amount: &str,
    server_key: &str,
    signature_key: &str,
) -> bool {
    let mut hasher = Sha512::new();
    hasher.update(order_id.as_bytes());
    hasher.update(status_code.as_bytes());
    hasher.update(gross_amount.as_bytes());
    hasher.update(server_key.as_bytes());
    hex::encode(hasher.finalize()).eq_ignore_ascii_case(signature_key.trim())
}

/// Pembulatan per-SKU ke rupiah terdekat (keputusan terkunci): fraksi desimal
/// lebih dari 5 ke atas, tepat-5 ikut ke bawah (19995,6 -> 19996).
pub fn round_to_rupiah(value: f64) -> i64 {
    let floor = value.floor();
    let frac = value - floor;
    if frac - 0.5 > 1e-9 {
        floor as i64 + 1
    } else {
        floor as i64
    }
}

/// `order_id` format `PF-{POS|ONL}-{uuid8}`, charset `[A-Za-z0-9-_.~]`,
/// maks 50 char (batas Midtrans).
pub fn generate_order_id(source_type: &str) -> String {
    let tag = if source_type == SOURCE_ONLINE {
        "ONL"
    } else {
        "POS"
    };
    let suffix: String = Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(8)
        .collect::<String>()
        .to_uppercase();
    format!("PF-{tag}-{suffix}")
}

pub fn validate_order_id(order_id: &str) -> AppResult<()> {
    if order_id.trim().is_empty() {
        return Err(AppError::Validation("order_id is required".to_string()));
    }
    if order_id.len() > 50 {
        return Err(AppError::Validation(
            "order_id must not exceed 50 characters".to_string(),
        ));
    }
    let valid = order_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~'));
    if !valid {
        return Err(AppError::Validation(
            "order_id contains invalid characters".to_string(),
        ));
    }
    Ok(())
}

fn normalize_source_type(source_type: Option<String>) -> AppResult<String> {
    match source_type.as_deref().map(str::trim) {
        None | Some("") | Some(SOURCE_POS) => Ok(SOURCE_POS.to_string()),
        Some(SOURCE_ONLINE) => Ok(SOURCE_ONLINE.to_string()),
        Some(other) => Err(AppError::Validation(format!(
            "source_type must be '{SOURCE_POS}' or '{SOURCE_ONLINE}', got '{other}'"
        ))),
    }
}

pub fn to_response(payment: &MidtransPayment) -> MidtransPaymentResponse {
    MidtransPaymentResponse {
        order_id: payment.order_id.clone(),
        source_type: payment.source_type.clone(),
        gross_amount: payment.gross_amount as i64,
        transaction_status: payment.transaction_status.clone(),
        fraud_status: payment.fraud_status.clone(),
        payment_type: payment.payment_type.clone(),
        snap_token: payment.snap_token.clone(),
        redirect_url: payment.redirect_url.clone(),
        sale_id: payment.sale_id,
        online_order_id: payment.online_order_id,
        expires_at: payment.expires_at,
        settled_at: payment.settled_at,
    }
}

fn api_base_url(settings: &MidtransSettings) -> &'static str {
    if settings.is_production {
        "https://api.midtrans.com"
    } else {
        "https://api.sandbox.midtrans.com"
    }
}

/// Status ternormalisasi dari notifikasi maupun GET Status API, siap diproses
/// oleh [`apply_status_update`].
pub struct StatusUpdate {
    pub transaction_status: String,
    pub fraud_status: Option<String>,
    pub payment_type: Option<String>,
    pub va_info: Option<Value>,
    pub raw: Value,
}

fn va_info_from_notification(payload: &MidtransNotificationPayload) -> Option<Value> {
    let mut info = serde_json::Map::new();
    if let Some(va_numbers) = &payload.va_numbers {
        info.insert(
            "va_numbers".to_string(),
            json!(
                va_numbers
                    .iter()
                    .map(|va| json!({ "bank": va.bank, "va_number": va.va_number }))
                    .collect::<Vec<_>>()
            ),
        );
    }
    if let Some(va) = &payload.permata_va_number {
        info.insert("permata_va_number".to_string(), json!(va));
    }
    if payload.bill_key.is_some() || payload.biller_code.is_some() {
        info.insert(
            "bill".to_string(),
            json!({ "bill_key": payload.bill_key, "biller_code": payload.biller_code }),
        );
    }
    if info.is_empty() {
        None
    } else {
        Some(Value::Object(info))
    }
}

fn str_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn status_update_from_status_api(body: &Value) -> AppResult<StatusUpdate> {
    let transaction_status = str_field(body, "transaction_status").ok_or_else(|| {
        AppError::Internal("midtrans status response misses transaction_status".to_string())
    })?;
    Ok(StatusUpdate {
        transaction_status,
        fraud_status: str_field(body, "fraud_status"),
        payment_type: str_field(body, "payment_type"),
        va_info: body.get("va_numbers").cloned().or_else(|| {
            body.get("permata_va_number")
                .map(|va| json!({ "permata_va_number": va }))
        }),
        raw: body.clone(),
    })
}

async fn snap_create_transaction(
    settings: &MidtransSettings,
    order_id: &str,
    gross_amount: i64,
    items: &[(String, String, i64)],
    expiry_minutes: u64,
) -> AppResult<(Option<String>, Option<String>)> {
    let item_details: Vec<Value> = items
        .iter()
        .map(|(id, name, price)| {
            // Kuantitas fraksional tidak didukung Midtrans: satu baris per SKU
            // dengan quantity 1 dan harga = total baris yang sudah dibulatkan.
            json!({ "id": id, "price": price, "quantity": 1, "name": name })
        })
        .collect();
    let body = json!({
        "transaction_details": { "order_id": order_id, "gross_amount": gross_amount },
        "item_details": item_details,
        "expiry": { "unit": "minute", "duration": expiry_minutes },
        "enabled_payments": ENABLED_PAYMENTS,
    });
    let url = format!(
        "{}/transactions",
        settings.snap_base_url.trim_end_matches('/')
    );

    let response = reqwest::Client::new()
        .post(&url)
        .basic_auth(&settings.server_key, Some(""))
        .json(&body)
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("midtrans snap request failed: {error}")))?;
    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        return Err(AppError::Internal(format!(
            "midtrans snap rejected the charge (status {status}): {detail}"
        )));
    }
    let data: Value = response
        .json()
        .await
        .map_err(|error| AppError::Internal(format!("invalid snap response: {error}")))?;
    Ok((str_field(&data, "token"), str_field(&data, "redirect_url")))
}

async fn snap_get_status(settings: &MidtransSettings, order_id: &str) -> AppResult<StatusUpdate> {
    let url = format!("{}/v2/{order_id}/status", api_base_url(settings));
    let response = reqwest::Client::new()
        .get(&url)
        .basic_auth(&settings.server_key, Some(""))
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("midtrans status request failed: {error}")))?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(AppError::NotFound);
    }
    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        return Err(AppError::Internal(format!(
            "midtrans status request failed (status {status}): {detail}"
        )));
    }
    let data: Value = response
        .json()
        .await
        .map_err(|error| AppError::Internal(format!("invalid status response: {error}")))?;
    status_update_from_status_api(&data)
}

/// Buat charge Snap pending: validasi stok/ketersediaan + hitung subtotal
/// TANPA mengurangi stok, panggil Midtrans, simpan baris pending.
pub async fn create_charge(
    pool: &PgPool,
    settings: &MidtransSettings,
    user: &AuthUser,
    request: CreateMidtransChargeRequest,
) -> AppResult<MidtransChargeResponse> {
    if request.items.is_empty() {
        return Err(AppError::Validation("items must not be empty".to_string()));
    }
    // Gerbang lintas-mode kasir: charge Midtrans hanya sah saat toko dalam
    // mode `midtrans`. Mode `manual` memakai pencatatan transfer manual,
    // sehingga request Snap lintas-mode ditolak server dengan error jelas.
    if repository::get_payment_mode(pool).await? != PAYMENT_MODE_MIDTRANS {
        return Err(AppError::Validation(
            "midtrans payments are not available in manual payment mode".to_string(),
        ));
    }
    let source_type = normalize_source_type(request.source_type)?;
    if source_type == SOURCE_ONLINE {
        let online_order_id = request.online_order_id.ok_or_else(|| {
            AppError::Validation("online_order_id is required for online charges".to_string())
        })?;
        if !repository::order_exists(pool, online_order_id).await? {
            return Err(AppError::NotFound);
        }
    }
    let member_id = documents::validate_active_member(pool, request.member_id).await?;
    for item in &request.items {
        if item.quantity <= 0.0 {
            return Err(AppError::Validation(
                "quantity must be greater than 0".to_string(),
            ));
        }
    }

    // Validasi sesi + snapshot SKU (baca saja, tanpa kunci stok).
    let mut validation_tx = pool.begin().await?;
    cash_registers::ensure_open_cash_register_session(
        &mut validation_tx,
        request.cash_register_session_id,
    )
    .await?;
    let mut prepared = Vec::new();
    for item in &request.items {
        let product =
            documents::load_product_sku_snapshot(&mut validation_tx, item.product_sku_id, false)
                .await?;
        if product.stock < item.quantity {
            return Err(AppError::Conflict(format!(
                "insufficient stock for product SKU {}",
                product.sku
            )));
        }
        prepared.push((item.quantity, product));
    }
    validation_tx.commit().await?;

    // Nominal Snap: jumlah total baris per-SKU yang sudah dibulatkan.
    let mut gross_amount: i64 = 0;
    let mut snap_items = Vec::new();
    let mut internal_subtotal = 0.0;
    for (quantity, product) in &prepared {
        let line_internal = *quantity * product.selling_price;
        internal_subtotal += line_internal;
        let line_rounded = round_to_rupiah(line_internal);
        if line_rounded < 0 {
            return Err(AppError::Validation(
                "rounded line amount must not be negative".to_string(),
            ));
        }
        gross_amount += line_rounded;
        snap_items.push((
            product.sku.clone(),
            product.product_name.clone(),
            line_rounded,
        ));
    }
    if gross_amount > i64::from(i32::MAX) {
        return Err(AppError::Validation(
            "gross_amount exceeds the maximum supported value".to_string(),
        ));
    }

    let order_id = generate_order_id(&source_type);
    let expires_at = Utc::now() + chrono::Duration::minutes(settings.payment_expiry_minutes as i64);
    let (snap_token, redirect_url) = snap_create_transaction(
        settings,
        &order_id,
        gross_amount,
        &snap_items,
        settings.payment_expiry_minutes,
    )
    .await?;

    let stored_items = request
        .items
        .iter()
        .map(|item| json!({ "product_sku_id": item.product_sku_id, "quantity": item.quantity }))
        .collect::<Vec<_>>();
    let mut tx = pool.begin().await?;
    // Sesi bisa tutup saat memanggil Snap: pastikan lagi sebelum menyimpan.
    cash_registers::ensure_open_cash_register_session(&mut tx, request.cash_register_session_id)
        .await?;
    repository::insert_payment(
        &mut tx,
        &order_id,
        &source_type,
        gross_amount,
        snap_token.as_deref(),
        redirect_url.as_deref(),
        &json!(stored_items),
        member_id,
        request.notes.as_deref(),
        request.online_order_id,
        Some(expires_at),
        Some(user.id),
    )
    .await?;
    tx.commit().await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(user.id),
            "midtrans_payment.charge",
            "midtrans_payment",
            None,
        )
        .with_metadata(json!({
            "order_id": order_id,
            "source_type": source_type,
            "gross_amount": gross_amount,
            "internal_subtotal": internal_subtotal,
            "rounding_difference": gross_amount as f64 - internal_subtotal,
            "cash_register_session_id": request.cash_register_session_id,
        })),
    )
    .await;

    Ok(MidtransChargeResponse {
        order_id,
        snap_token,
        redirect_url,
        gross_amount,
        expires_at: Some(expires_at),
    })
}

/// Satu baris pesanan online untuk charge Snap: harga dari snapshot checkout,
/// nominal per-SKU dibulatkan ke rupiah saat kirim ke Midtrans.
#[derive(Debug, Clone)]
pub struct OnlineChargeItem {
    pub product_sku_id: i64,
    pub quantity: f64,
    pub unit_price: f64,
    pub sku: String,
    pub name: String,
}

/// Susun `item_details` Snap + `gross_amount` dari pesanan online (murni,
/// tanpa database) agar bisa diuji unit. Per-SKU dibulatkan via
/// [`round_to_rupiah`]; ongkos kirim menjadi baris penyesuai tersendiri bila
/// lebih dari nol sehingga jumlah baris selalu sama dengan `gross_amount`.
pub fn build_online_snap_items(
    items: &[OnlineChargeItem],
    delivery_fee: f64,
) -> (Vec<(String, String, i64)>, i64) {
    let mut snap_items = Vec::with_capacity(items.len() + 1);
    let mut gross_amount: i64 = 0;
    for item in items {
        let price = round_to_rupiah(item.quantity * item.unit_price);
        gross_amount += price;
        snap_items.push((item.sku.clone(), item.name.clone(), price));
    }
    let delivery = round_to_rupiah(delivery_fee);
    if delivery > 0 {
        gross_amount += delivery;
        snap_items.push((
            "delivery-fee".to_string(),
            "Ongkos kirim".to_string(),
            delivery,
        ));
    }
    (snap_items, gross_amount)
}

/// Buat charge Snap untuk checkout publik online: tanpa sesi kasir dan tanpa
/// `AuthUser` (pembeli bisa guest/member). Pesanan sudah dibuat + stok sudah
/// divalidasi di `online_orders::checkout` sebelum fungsi ini dipanggil;
/// tautan balik disimpan di sisi payments (`midtrans_payments.online_order_id`)
/// tanpa migrasi ke `online_orders`. Stok tetap belum berkurang — baru saat
/// settlement via [`apply_notification`].
pub async fn create_online_charge(
    pool: &PgPool,
    settings: &MidtransSettings,
    online_order_id: i64,
    member_id: Option<i64>,
    notes: Option<&str>,
    items: &[OnlineChargeItem],
    delivery_fee: f64,
) -> AppResult<MidtransChargeResponse> {
    if items.is_empty() {
        return Err(AppError::Validation("items must not be empty".to_string()));
    }
    for item in items {
        if !item.quantity.is_finite() || item.quantity <= 0.0 {
            return Err(AppError::Validation(
                "quantity must be greater than 0".to_string(),
            ));
        }
        if !item.unit_price.is_finite() || item.unit_price < 0.0 {
            return Err(AppError::Validation(
                "unit_price must not be negative".to_string(),
            ));
        }
    }
    if !delivery_fee.is_finite() || delivery_fee < 0.0 {
        return Err(AppError::Validation(
            "delivery_fee must not be negative".to_string(),
        ));
    }
    if !repository::order_exists(pool, online_order_id).await? {
        return Err(AppError::NotFound);
    }

    let (snap_items, gross_amount) = build_online_snap_items(items, delivery_fee);
    if gross_amount > i64::from(i32::MAX) {
        return Err(AppError::Validation(
            "gross_amount exceeds the maximum supported value".to_string(),
        ));
    }

    let order_id = generate_order_id(SOURCE_ONLINE);
    let expires_at = Utc::now() + chrono::Duration::minutes(settings.payment_expiry_minutes as i64);
    let (snap_token, redirect_url) = snap_create_transaction(
        settings,
        &order_id,
        gross_amount,
        &snap_items,
        settings.payment_expiry_minutes,
    )
    .await?;

    let stored_items = items
        .iter()
        .map(|item| json!({ "product_sku_id": item.product_sku_id, "quantity": item.quantity }))
        .collect::<Vec<_>>();
    let mut tx = pool.begin().await?;
    repository::insert_payment(
        &mut tx,
        &order_id,
        SOURCE_ONLINE,
        gross_amount,
        snap_token.as_deref(),
        redirect_url.as_deref(),
        &json!(stored_items),
        member_id,
        notes,
        Some(online_order_id),
        Some(expires_at),
        None,
    )
    .await?;
    tx.commit().await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(None, "midtrans_payment.charge", "midtrans_payment", None)
            .with_metadata(json!({
                "order_id": order_id,
                "source_type": SOURCE_ONLINE,
                "gross_amount": gross_amount,
                "online_order_id": online_order_id,
            })),
    )
    .await;

    Ok(MidtransChargeResponse {
        order_id,
        snap_token,
        redirect_url,
        gross_amount,
        expires_at: Some(expires_at),
    })
}

/// Terapkan satu pembaruan status (dari webhook terverifikasi maupun GET
/// Status rekonsiliasi) dalam satu transaksi DB: update baris pembayaran dan,
/// hanya pada transisi lunas pertama (`settled_at IS NULL`), jalankan efek
/// settlement (sale + stok untuk POS, tandai lunas untuk online).
/// Ringkasan efek settlement untuk audit pasca-commit.
pub struct SettlementSummary {
    pub sale_id: Option<i64>,
    pub subtotal: Option<f64>,
    pub session_id: Option<i64>,
}

async fn apply_status_update(
    tx: &mut Transaction<'_, Postgres>,
    payment: &MidtransPayment,
    update: &StatusUpdate,
) -> AppResult<(NotificationAction, Option<SettlementSummary>)> {
    let action = decide_notification_action(
        &payment.transaction_status,
        payment.fraud_status.as_deref(),
        payment.settled_at.is_some(),
        &update.transaction_status,
        update.fraud_status.as_deref(),
    );
    match action {
        NotificationAction::Ignore => Ok((action, None)),
        NotificationAction::UpdateStatus => {
            repository::update_status(
                tx,
                &payment.order_id,
                &update.transaction_status,
                update.fraud_status.as_deref(),
                update.payment_type.as_deref(),
                update.va_info.as_ref(),
                &update.raw,
            )
            .await?;
            Ok((action, None))
        }
        NotificationAction::Settle => {
            let summary = settle_payment(tx, payment, update).await?;
            Ok((action, Some(summary)))
        }
    }
}

async fn settle_payment(
    tx: &mut Transaction<'_, Postgres>,
    payment: &MidtransPayment,
    update: &StatusUpdate,
) -> AppResult<SettlementSummary> {
    // Charge online dibuat dari checkout publik tanpa kasir, jadi created_by
    // boleh None (processed_by NULL = dilunasi otomatis oleh sistem).
    // Charge POS selalu dibuat kasir terautentikasi sehingga Some.
    if payment.source_type == SOURCE_ONLINE {
        if let Some(online_order_id) = payment.online_order_id {
            let order = crate::modules::online_orders::repository::lock_order(tx, online_order_id)
                .await?
                .ok_or(AppError::NotFound)?;
            if order.payment_status != "paid" {
                crate::modules::online_orders::repository::set_order_paid(
                    tx,
                    online_order_id,
                    payment.created_by,
                )
                .await?;
            }
        }
        // Stok online dipotong jalur fulfilment (pack) yang sudah ada,
        // bukan di sini — konsisten dengan pesanan manual.
    } else {
        // Charge POS selalu dibuat kasir terautentikasi: created_by wajib ada.
        let cashier_id = payment.created_by.ok_or_else(|| {
            AppError::Internal("midtrans charge misses the cashier who created it".to_string())
        })?;
        // Keputusan terkunci: settlement memakai sesi yang SEDANG TERBUKA,
        // bukan sesi asal charge. Tanpa sesi terbuka, batalkan transaksi
        // (Midtrans akan retry) agar sale tidak yatim.
        let session_id = repository::find_open_session(tx).await?.ok_or_else(|| {
            AppError::Conflict("no open cash register session for midtrans settlement".to_string())
        })?;
        // Member divalidasi ulang: bisa dinonaktifkan antara charge dan lunas.
        let member_active: Option<bool> = match payment.member_id {
            Some(member_id) => {
                sqlx::query_scalar::<_, bool>("SELECT is_active FROM members WHERE id = $1")
                    .bind(member_id)
                    .fetch_optional(&mut **tx)
                    .await?
            }
            None => None,
        };
        let member_id = match (payment.member_id, member_active) {
            (Some(_), Some(true)) => payment.member_id,
            (Some(_), Some(false)) => {
                return Err(AppError::Validation("member is not active".to_string()));
            }
            (Some(_), None) => return Err(AppError::NotFound),
            (None, _) => None,
        };
        let items: Vec<SaleItemInput> = repository::stored_items(payment)
            .into_iter()
            .map(|item| SaleItemInput {
                product_sku_id: item.product_sku_id,
                quantity: item.quantity,
            })
            .collect();
        // `paid_amount = None`: lunas pas. Selisih pembulatan rupiah
        // (gross Snap vs subtotal DB) tercatat di audit di bawah. Sale
        // settlement selalu non-tunai `midtrans` dengan referensi order_id
        // agar laporan bisa memisahkan omzet tunai vs non-tunai.
        let (sale_id, subtotal) = documents::create_sale_tx(
            tx,
            cashier_id,
            session_id,
            member_id,
            None,
            None,
            payment.notes.clone(),
            items,
            documents::SALE_PAYMENT_MIDTRANS,
            Some(payment.order_id.clone()),
        )
        .await?;
        repository::update_status(
            tx,
            &payment.order_id,
            &update.transaction_status,
            update.fraud_status.as_deref(),
            update.payment_type.as_deref(),
            update.va_info.as_ref(),
            &update.raw,
        )
        .await?;
        let settled = repository::mark_settled(tx, &payment.order_id, Some(sale_id)).await?;
        if !settled {
            return Err(AppError::Conflict(
                "midtrans payment was settled concurrently".to_string(),
            ));
        }
        return Ok(SettlementSummary {
            sale_id: Some(sale_id),
            subtotal: Some(subtotal),
            session_id: Some(session_id),
        });
    }
    repository::update_status(
        tx,
        &payment.order_id,
        &update.transaction_status,
        update.fraud_status.as_deref(),
        update.payment_type.as_deref(),
        update.va_info.as_ref(),
        &update.raw,
    )
    .await?;
    let settled = repository::mark_settled(tx, &payment.order_id, None).await?;
    if !settled {
        return Err(AppError::Conflict(
            "midtrans payment was settled concurrently".to_string(),
        ));
    }
    Ok(SettlementSummary {
        sale_id: None,
        subtotal: None,
        session_id: None,
    })
}

fn settlement_audit_record(
    payment: &MidtransPayment,
    sale_id: Option<i64>,
    subtotal: Option<f64>,
    session_id: Option<i64>,
) -> audit::AuditLogRecord {
    audit::AuditLogRecord::success(
        payment.created_by,
        "midtrans_payment.settle",
        "midtrans_payment",
        Some(payment.id),
    )
    .with_metadata(json!({
        "order_id": payment.order_id,
        "source_type": payment.source_type,
        "gross_amount": payment.gross_amount,
        "subtotal_amount": subtotal,
        "rounding_difference": subtotal.map(|sub| payment.gross_amount as f64 - sub),
        "sale_id": sale_id,
        "online_order_id": payment.online_order_id,
        "cash_register_session_id": session_id,
    }))
}

/// Webhook publik Midtrans: verifikasi signature DULU, lalu terapkan idempoten.
/// Selalu 200 untuk notifikasi valid yang tidak butuh retry (duplikat,
/// terminal, out-of-order); error hanya untuk signature invalid, order tak
/// dikenal, atau settlement yang belum bisa dibuat (agar Midtrans retry).
pub async fn apply_notification(
    pool: &PgPool,
    settings: &MidtransSettings,
    payload: &MidtransNotificationPayload,
) -> AppResult<MidtransPaymentResponse> {
    validate_order_id(&payload.order_id)?;
    if !verify_signature(
        &payload.order_id,
        &payload.status_code,
        &payload.gross_amount,
        &settings.server_key,
        &payload.signature_key,
    ) {
        return Err(AppError::Unauthorized);
    }
    let raw = serde_json::to_value(payload)
        .map_err(|error| AppError::Internal(format!("cannot serialize notification: {error}")))?;
    let update = StatusUpdate {
        transaction_status: payload.transaction_status.clone(),
        fraud_status: payload.fraud_status.clone(),
        payment_type: payload.payment_type.clone(),
        va_info: va_info_from_notification(payload),
        raw,
    };
    apply_update(pool, &payload.order_id, &update).await
}

async fn apply_update(
    pool: &PgPool,
    order_id: &str,
    update: &StatusUpdate,
) -> AppResult<MidtransPaymentResponse> {
    let mut tx = pool.begin().await?;
    let payment = repository::lock_by_order_id(&mut tx, order_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let (action, settlement) = apply_status_update(&mut tx, &payment, update).await?;
    tx.commit().await?;

    audit::record_audit_log_best_effort(
        pool,
        match action {
            NotificationAction::Settle => {
                let summary = settlement.unwrap_or(SettlementSummary {
                    sale_id: None,
                    subtotal: None,
                    session_id: None,
                });
                settlement_audit_record(
                    &payment,
                    summary.sale_id,
                    summary.subtotal,
                    summary.session_id,
                )
            }
            NotificationAction::UpdateStatus => {
                let terminal =
                    is_terminal_status(&update.transaction_status, update.fraud_status.as_deref());
                audit::AuditLogRecord::success(
                    payment.created_by,
                    if terminal {
                        "midtrans_payment.expire"
                    } else {
                        "midtrans_payment.notification"
                    },
                    "midtrans_payment",
                    Some(payment.id),
                )
                .with_metadata(json!({
                    "order_id": payment.order_id,
                    "action": if terminal { "terminal" } else { "update" },
                    "transaction_status": update.transaction_status,
                }))
            }
            NotificationAction::Ignore => audit::AuditLogRecord::success(
                payment.created_by,
                "midtrans_payment.notification",
                "midtrans_payment",
                Some(payment.id),
            )
            .with_metadata(json!({
                "order_id": payment.order_id,
                "action": "ignored",
                "transaction_status": update.transaction_status,
            })),
        },
    )
    .await;

    repository::find_by_order_id(pool, order_id)
        .await?
        .ok_or(AppError::NotFound)
        .map(|payment| to_response(&payment))
}

/// Polling kasir: baca baris pembayaran; `refresh=true` menantang Midtrans
/// (GET Status API, server-to-server terotentikasi) lalu menerapkan hasilnya
/// lewat jalur idempoten yang sama dengan webhook.
pub async fn get_payment(
    pool: &PgPool,
    settings: Option<&MidtransSettings>,
    order_id: &str,
    refresh: bool,
) -> AppResult<MidtransPaymentResponse> {
    validate_order_id(order_id)?;
    if refresh {
        let settings =
            settings.ok_or_else(|| AppError::Internal("midtrans is not configured".to_string()))?;
        let payment = repository::find_by_order_id(pool, order_id)
            .await?
            .ok_or(AppError::NotFound)?;
        if payment.settled_at.is_none()
            && !is_terminal_status(&payment.transaction_status, payment.fraud_status.as_deref())
        {
            let update = snap_get_status(settings, order_id).await?;
            return apply_update(pool, order_id, &update).await;
        }
    }
    repository::find_by_order_id(pool, order_id)
        .await?
        .ok_or(AppError::NotFound)
        .map(|payment| to_response(&payment))
}

pub fn require_settings(settings: Option<&MidtransSettings>) -> AppResult<&MidtransSettings> {
    settings.ok_or_else(|| AppError::Internal("midtrans is not configured".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha512};

    fn signature(order_id: &str, status_code: &str, gross: &str, key: &str) -> String {
        let mut hasher = Sha512::new();
        hasher.update(order_id.as_bytes());
        hasher.update(status_code.as_bytes());
        hasher.update(gross.as_bytes());
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    #[test]
    fn signature_verification_accepts_valid_key() {
        let expected = signature("PF-POS-ABCDEF12", "200", "50000", "server-secret");
        assert!(verify_signature(
            "PF-POS-ABCDEF12",
            "200",
            "50000",
            "server-secret",
            &expected
        ));
        // Perbandingan case-insensitive (hex huruf besar tetap valid).
        assert!(verify_signature(
            "PF-POS-ABCDEF12",
            "200",
            "50000",
            "server-secret",
            &expected.to_uppercase()
        ));
    }

    #[test]
    fn signature_verification_rejects_tampered_fields() {
        let expected = signature("PF-POS-ABCDEF12", "200", "50000", "server-secret");
        assert!(!verify_signature(
            "PF-POS-ABCDEF12",
            "200",
            "50001",
            "server-secret",
            &expected
        ));
        assert!(!verify_signature(
            "PF-POS-ABCDEF12",
            "200",
            "50000",
            "wrong-secret",
            &expected
        ));
        assert!(!verify_signature(
            "PF-POS-OTHER123",
            "200",
            "50000",
            "server-secret",
            &expected
        ));
    }

    #[test]
    fn notification_settles_once_then_ignores_duplicates() {
        // Transisi lunas pertama dari pending.
        assert_eq!(
            decide_notification_action("pending", None, false, "settlement", None),
            NotificationAction::Settle
        );
        // Duplikat settlement setelah `settled_at` terkunci: abaikan.
        assert_eq!(
            decide_notification_action("settlement", None, true, "settlement", None),
            NotificationAction::Ignore
        );
        assert_eq!(
            decide_notification_action("pending", None, true, "settlement", None),
            NotificationAction::Ignore
        );
    }

    #[test]
    fn notification_ignores_terminal_and_out_of_order() {
        // Status terminal tidak bisa diubah notifikasi apa pun.
        for stored in [
            "settlement",
            "capture",
            "expire",
            "cancel",
            "deny",
            "failure",
        ] {
            let fraud = if stored == "capture" {
                Some("accept")
            } else {
                None
            };
            assert_eq!(
                decide_notification_action(stored, fraud, false, "settlement", None),
                NotificationAction::Ignore,
                "stored {stored} must be terminal"
            );
        }
        // Out-of-order: pending datang setelah settlement tersimpan.
        assert_eq!(
            decide_notification_action("settlement", None, false, "pending", None),
            NotificationAction::Ignore
        );
        // `capture + challenge` belum terminal: settlement menyusul tetap lunas.
        assert_eq!(
            decide_notification_action("capture", Some("challenge"), false, "settlement", None),
            NotificationAction::Settle
        );
        // `capture + accept` langsung lunas.
        assert_eq!(
            decide_notification_action("pending", None, false, "capture", Some("accept")),
            NotificationAction::Settle
        );
        // `capture + challenge` hanya dicatat, tanpa settlement.
        assert_eq!(
            decide_notification_action("pending", None, false, "capture", Some("challenge")),
            NotificationAction::UpdateStatus
        );
        // Pending dan kegagalan wajar hanya update status.
        assert_eq!(
            decide_notification_action("pending", None, false, "pending", None),
            NotificationAction::UpdateStatus
        );
        assert_eq!(
            decide_notification_action("pending", None, false, "expire", None),
            NotificationAction::UpdateStatus
        );
        // Info pasca-settlement (refund) selalu diabaikan.
        assert_eq!(
            decide_notification_action("pending", None, false, "refund", None),
            NotificationAction::Ignore
        );
    }

    #[test]
    fn rounding_follows_locked_rule() {
        // Fraksi desimal > 5 ke atas, tepat-5 ikut ke bawah.
        assert_eq!(round_to_rupiah(19995.6), 19996);
        assert_eq!(round_to_rupiah(19995.5), 19995);
        assert_eq!(round_to_rupiah(19995.4), 19995);
        assert_eq!(round_to_rupiah(100.51), 101);
        assert_eq!(round_to_rupiah(100.50), 100);
        assert_eq!(round_to_rupiah(100.0), 100);
        assert_eq!(round_to_rupiah(0.0), 0);
    }

    fn online_item(sku_id: i64, quantity: f64, price: f64) -> OnlineChargeItem {
        OnlineChargeItem {
            product_sku_id: sku_id,
            quantity,
            unit_price: price,
            sku: format!("SKU-{sku_id}"),
            name: format!("Product {sku_id}"),
        }
    }

    #[test]
    fn online_snap_items_round_per_sku_and_sum_to_gross() {
        // Fraksi desimal > 5 ke atas, tepat-5 ikut ke bawah (aturan terkunci).
        let items = vec![
            online_item(1, 1.5, 13330.4), // 19995.6 -> 19996
            online_item(2, 2.0, 5000.25), // 10000.5 -> 10000
        ];
        let (snap, gross) = build_online_snap_items(&items, 0.0);
        assert_eq!(snap.len(), 2);
        assert_eq!(snap[0].2, 19996);
        assert_eq!(snap[1].2, 10000);
        assert_eq!(gross, 29996);
        assert_eq!(gross, snap.iter().map(|(_, _, price)| *price).sum::<i64>());
    }

    #[test]
    fn online_snap_items_add_delivery_fee_as_adjustment_line() {
        let items = vec![online_item(1, 1.0, 20000.0)];
        let (snap, gross) = build_online_snap_items(&items, 5000.0);
        assert_eq!(snap.len(), 2);
        assert_eq!(
            snap[1],
            ("delivery-fee".to_string(), "Ongkos kirim".to_string(), 5000)
        );
        assert_eq!(gross, 25000);

        // Tanpa ongkir: tidak ada baris penyesuai.
        let (snap, gross) = build_online_snap_items(&items, 0.0);
        assert_eq!(snap.len(), 1);
        assert_eq!(gross, 20000);
    }

    #[test]
    fn order_id_format_matches_midtrans_charset() {
        let pos = generate_order_id(SOURCE_POS);
        let online = generate_order_id(SOURCE_ONLINE);
        assert!(pos.starts_with("PF-POS-"));
        assert!(online.starts_with("PF-ONL-"));
        for order_id in [&pos, &online] {
            assert!(validate_order_id(order_id).is_ok());
            assert!(order_id.len() <= 50);
            assert!(
                order_id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~'))
            );
        }
        assert_ne!(pos, generate_order_id(SOURCE_POS));
        assert!(validate_order_id("").is_err());
        assert!(validate_order_id(&"x".repeat(51)).is_err());
        assert!(validate_order_id("PF-POS-bad id!").is_err());
    }
}
