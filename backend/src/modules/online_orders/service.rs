use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::MidtransSettings,
    error::{AppError, AppResult},
    infra::s3::{self, ObjectStorage},
    models::{ListQuery, MemberResponse, PaginationMeta, UploadResponse},
    modules::payments::service as payments_service,
    services::{audit, member_auth},
};

use super::{
    dto::{
        CheckoutItemResponse, CheckoutPaymentInfo, CheckoutRequest, CheckoutResponse,
        OnlineOrderDetailResponse, OnlineOrderItemResponse, OnlineOrderResponse,
        PublicCatalogQuery, PublicCategory, PublicProductItem, StoreInfoResponse,
        UpdateStoreSettingsRequest,
    },
    model::OrderRow,
    repository::{self, NewOrderItem},
};

pub const FULFILLMENT_PICKUP: &str = "pickup";
pub const FULFILLMENT_DELIVERY: &str = "delivery";
pub const PAYMENT_PAY_AT_STORE: &str = "pay_at_store";
pub const PAYMENT_TRANSFER: &str = "transfer";
pub const PAYMENT_MIDTRANS: &str = "midtrans";
pub const PAYMENT_MODE_MANUAL: &str = "manual";
pub const PAYMENT_MODE_MIDTRANS: &str = "midtrans";

/// Normalisasi mode pembayaran toko aktif. Hanya `midtrans` yang diakui;
/// nilai lain (termasuk baris belum ada atau data lama) berarti `manual`
/// agar toko yang sudah jalan tidak berubah perilaku.
pub fn normalize_payment_mode(value: Option<&str>) -> String {
    match value.map(str::trim) {
        Some(PAYMENT_MODE_MIDTRANS) => PAYMENT_MODE_MIDTRANS.to_string(),
        _ => PAYMENT_MODE_MANUAL.to_string(),
    }
}

const MAX_CHECKOUT_ITEMS: usize = 100;

#[derive(Debug)]
pub struct ValidatedItem {
    pub product_sku_id: i64,
    pub quantity: f64,
}

#[derive(Debug)]
pub struct ValidatedCheckout {
    pub customer_name: String,
    pub customer_phone: String,
    pub fulfillment_method: String,
    pub delivery_address: Option<String>,
    pub payment_method: String,
    pub notes: Option<String>,
    pub items: Vec<ValidatedItem>,
}

/// Validasi checkout murni (tanpa database) agar bisa diuji unit.
/// `payment_mode` adalah mode toko aktif yang sudah dinormalisasi
/// (`manual` | `midtrans`). Jalur non-tunai bersifat eksklusif mengikuti
/// mode toko; request lintas-mode ditolak server dengan error yang jelas:
/// - mode `manual`: terima `pay_at_store`/`transfer`, tolak `midtrans`;
/// - mode `midtrans`: terima `pay_at_store`/`midtrans`, tolak `transfer`.
pub fn validate_checkout(
    request: &CheckoutRequest,
    payment_mode: &str,
) -> Result<ValidatedCheckout, String> {
    let customer_name = request.customer_name.trim().to_string();
    if customer_name.is_empty() {
        return Err("customer_name is required".to_string());
    }
    if customer_name.len() > 120 {
        return Err("customer_name is too long (max 120)".to_string());
    }

    let customer_phone = request.customer_phone.trim().to_string();
    if customer_phone.is_empty() {
        return Err("customer_phone is required".to_string());
    }
    if customer_phone.len() > 30 {
        return Err("customer_phone is too long (max 30)".to_string());
    }

    let fulfillment_method = request.fulfillment_method.trim().to_string();
    if fulfillment_method != FULFILLMENT_PICKUP && fulfillment_method != FULFILLMENT_DELIVERY {
        return Err("fulfillment_method must be pickup or delivery".to_string());
    }

    let payment_method = request.payment_method.trim().to_string();
    match payment_mode.trim() {
        PAYMENT_MODE_MIDTRANS => {
            if payment_method != PAYMENT_PAY_AT_STORE && payment_method != PAYMENT_MIDTRANS {
                return Err(
                    "payment_method must be pay_at_store or midtrans in midtrans payment mode"
                        .to_string(),
                );
            }
        }
        _ => {
            if payment_method != PAYMENT_PAY_AT_STORE && payment_method != PAYMENT_TRANSFER {
                return Err(
                    "payment_method must be pay_at_store or transfer in manual payment mode"
                        .to_string(),
                );
            }
        }
    }

    let delivery_address = request
        .delivery_address
        .as_deref()
        .map(str::trim)
        .and_then(|s| {
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        });
    if fulfillment_method == FULFILLMENT_DELIVERY && delivery_address.is_none() {
        return Err("delivery_address is required for delivery orders".to_string());
    }

    if request.items.is_empty() {
        return Err("items must not be empty".to_string());
    }
    if request.items.len() > MAX_CHECKOUT_ITEMS {
        return Err(format!("items must not exceed {MAX_CHECKOUT_ITEMS}"));
    }

    let mut items = Vec::with_capacity(request.items.len());
    for item in &request.items {
        if item.product_sku_id <= 0 {
            return Err("product_sku_id must be positive".to_string());
        }
        if !item.quantity.is_finite() || item.quantity <= 0.0 {
            return Err("quantity must be greater than zero".to_string());
        }
        items.push(ValidatedItem {
            product_sku_id: item.product_sku_id,
            quantity: item.quantity,
        });
    }

    let notes = request.notes.as_deref().map(str::trim).and_then(|s| {
        if s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    });

    Ok(ValidatedCheckout {
        customer_name,
        customer_phone,
        fulfillment_method,
        delivery_address,
        payment_method,
        notes,
        items,
    })
}

/// Object key S3 untuk upload gambar. Murni agar bisa diuji unit.
pub fn build_object_key(prefix: &str, scope: &str, extension: &str) -> String {
    format!("{prefix}/{scope}/{}.{}", Uuid::new_v4(), extension)
}

/// Normalisasi nomor WhatsApp toko ke digit internasional (62...). Murni.
pub fn normalize_whatsapp(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if let Some(rest) = digits.strip_prefix('0') {
        return format!("62{rest}");
    }
    digits
}

pub async fn list_catalog(
    pool: &PgPool,
    s3: Option<&ObjectStorage>,
    query: PublicCatalogQuery,
) -> AppResult<(Vec<PublicProductItem>, PaginationMeta)> {
    repository::list_catalog(pool, &query, s3.map(ObjectStorage::base_url)).await
}

pub async fn list_categories(pool: &PgPool) -> AppResult<Vec<PublicCategory>> {
    repository::list_public_categories(pool).await
}

pub async fn get_store_info(
    pool: &PgPool,
    s3: Option<&ObjectStorage>,
) -> AppResult<StoreInfoResponse> {
    let store_name = repository::get_setting(pool, "store_name")
        .await?
        .unwrap_or_default();
    let operating_hours = repository::get_setting(pool, "operating_hours")
        .await?
        .unwrap_or_default();
    let bank_accounts = repository::get_setting(pool, "bank_accounts")
        .await?
        .unwrap_or_default();
    let ewallet_numbers = repository::get_setting(pool, "ewallet_numbers")
        .await?
        .unwrap_or_default();
    let qris_key = repository::get_setting(pool, "qris_image_key").await?;
    let delivery_fee: f64 = repository::get_setting(pool, "delivery_fee")
        .await?
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0.0);

    let qris_image_url = qris_key
        .as_deref()
        .filter(|k| !k.is_empty())
        .and_then(|key| s3.map(|storage| storage.public_url(key)));
    let shop_whatsapp = repository::get_setting(pool, "shop_whatsapp")
        .await?
        .map(|v| normalize_whatsapp(&v))
        .unwrap_or_default();
    // Hanya dua nilai yang sah; nilai lain (mis. data lama) dinormalkan ke manual.
    let payment_mode = normalize_payment_mode(
        repository::get_setting(pool, "payment_mode")
            .await?
            .as_deref(),
    );

    Ok(StoreInfoResponse {
        store_name,
        operating_hours,
        bank_accounts,
        ewallet_numbers,
        qris_image_url,
        delivery_fee,
        shop_whatsapp,
        payment_mode,
    })
}

/// Identitas pembeli hasil resolusi checkout: member teratribusi memakai
/// profil (nama/telepon body diabaikan — anti-spoof), guest memakai body.
#[derive(Debug, PartialEq)]
pub struct ResolvedCheckoutIdentity {
    pub customer_name: String,
    pub customer_phone: String,
    pub delivery_address: Option<String>,
    pub member_id: Option<i64>,
}

fn normalize_address(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Tentukan identitas pembeli sebelum validasi. Murni (tanpa database)
/// agar bisa diuji unit.
pub fn resolve_checkout_identity(
    request: &CheckoutRequest,
    member: Option<&MemberResponse>,
) -> ResolvedCheckoutIdentity {
    match member {
        Some(member) => ResolvedCheckoutIdentity {
            customer_name: member.full_name.clone(),
            customer_phone: member.phone.clone(),
            // Alamat body hanya override; default dari profil member.
            delivery_address: normalize_address(request.delivery_address.as_deref())
                .or_else(|| normalize_address(member.address.as_deref())),
            member_id: Some(member.id),
        },
        None => ResolvedCheckoutIdentity {
            customer_name: request.customer_name.trim().to_string(),
            customer_phone: request.customer_phone.trim().to_string(),
            delivery_address: normalize_address(request.delivery_address.as_deref()),
            member_id: None,
        },
    }
}

pub async fn checkout(
    pool: &PgPool,
    s3: Option<&ObjectStorage>,
    midtrans: Option<&MidtransSettings>,
    request: CheckoutRequest,
    member_id: Option<i64>,
) -> AppResult<CheckoutResponse> {
    // Token member → profil (wajib aktif, bila tidak 401). Tanpa token → guest.
    let member = match member_id {
        Some(id) => Some(member_auth::find_active_member(pool, id).await?),
        None => None,
    };
    let identity = resolve_checkout_identity(&request, member.as_ref());
    // Flag mode pembayaran dibaca di awal agar penolakan lintas-mode
    // terjadi server-side, bukan hanya disembunyikan di UI.
    let payment_mode = normalize_payment_mode(
        repository::get_setting(pool, "payment_mode")
            .await?
            .as_deref(),
    );
    // Validasi berjalan di atas identitas efektif agar aturan alamat/nama
    // tetap sama untuk member maupun guest.
    let effective = CheckoutRequest {
        customer_name: identity.customer_name,
        customer_phone: identity.customer_phone,
        fulfillment_method: request.fulfillment_method,
        delivery_address: identity.delivery_address,
        payment_method: request.payment_method,
        notes: request.notes,
        items: request.items,
    };
    let order = validate_checkout(&effective, &payment_mode).map_err(AppError::Validation)?;

    let sku_ids: Vec<i64> = order.items.iter().map(|i| i.product_sku_id).collect();
    let sku_rows = repository::load_checkout_skus(pool, &sku_ids).await?;

    let mut items = Vec::with_capacity(order.items.len());
    let mut subtotal = 0.0;
    for item in &order.items {
        let sku = sku_rows
            .iter()
            .find(|row| row.product_sku_id == item.product_sku_id)
            .ok_or_else(|| {
                AppError::Validation(format!(
                    "product SKU {} is not available",
                    item.product_sku_id
                ))
            })?;
        if !sku.is_active {
            return Err(AppError::Validation(format!(
                "{} is not available",
                sku.product_sku
            )));
        }
        if sku.stock < item.quantity {
            return Err(AppError::Validation(format!(
                "{} only has {} {} in stock",
                sku.product_name, sku.stock, sku.unit
            )));
        }
        let line_total = item.quantity * sku.selling_price;
        subtotal += line_total;
        items.push(NewOrderItem {
            product_id: sku.product_id,
            product_sku_id: sku.product_sku_id,
            product_name: sku.product_name.clone(),
            product_sku: sku.product_sku.clone(),
            unit: sku.unit.clone(),
            quantity: item.quantity,
            unit_price: sku.selling_price,
        });
    }

    let mut delivery_fee = 0.0;
    if order.fulfillment_method == FULFILLMENT_DELIVERY {
        delivery_fee = repository::get_setting(pool, "delivery_fee")
            .await?
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(0.0);
    }
    let total = subtotal + delivery_fee;

    let response_items: Vec<CheckoutItemResponse> = items
        .iter()
        .map(|item| CheckoutItemResponse {
            product_sku: item.product_sku.clone(),
            product_name: item.product_name.clone(),
            quantity: item.quantity,
            unit_price: item.unit_price,
            line_total: item.quantity * item.unit_price,
        })
        .collect();

    let saved = repository::insert_order(
        pool,
        repository::NewOrderData {
            customer_name: order.customer_name.clone(),
            customer_phone: order.customer_phone.clone(),
            fulfillment_method: order.fulfillment_method.clone(),
            delivery_address: order.delivery_address.clone(),
            payment_method: order.payment_method.clone(),
            notes: order.notes.clone(),
            subtotal,
            delivery_fee,
            member_id: identity.member_id,
            items,
        },
    )
    .await?;

    // Metode midtrans: buat charge Snap publik (tanpa sesi kasir), tautan
    // balik disimpan di sisi payments (`midtrans_payments.online_order_id`).
    // Respons mengembalikan snap_token/redirect_url, bukan payment_info.
    if order.payment_method == PAYMENT_MIDTRANS {
        let settings =
            midtrans.ok_or_else(|| AppError::Internal("midtrans is not configured".to_string()))?;
        let mut charge_items = Vec::with_capacity(order.items.len());
        for item in &order.items {
            let sku = sku_rows
                .iter()
                .find(|row| row.product_sku_id == item.product_sku_id)
                .ok_or_else(|| {
                    AppError::Validation(format!(
                        "product SKU {} is not available",
                        item.product_sku_id
                    ))
                })?;
            charge_items.push(payments_service::OnlineChargeItem {
                product_sku_id: item.product_sku_id,
                quantity: item.quantity,
                unit_price: sku.selling_price,
                sku: sku.product_sku.clone(),
                name: sku.product_name.clone(),
            });
        }
        let charge = payments_service::create_online_charge(
            pool,
            settings,
            saved.id,
            identity.member_id,
            order.notes.as_deref(),
            &charge_items,
            delivery_fee,
        )
        .await?;
        return Ok(CheckoutResponse {
            order_id: saved.id,
            order_no: saved.order_no,
            fulfillment_method: order.fulfillment_method,
            payment_method: order.payment_method,
            total_amount: total,
            items: response_items,
            payment_info: None,
            snap_token: charge.snap_token,
            redirect_url: charge.redirect_url,
            midtrans_order_id: Some(charge.order_id),
        });
    }

    // Info pembayaran manual (rekening/e-wallet/QRIS) hanya dikembalikan
    // saat mode manual + metode transfer.
    let payment_info =
        if order.payment_method == PAYMENT_TRANSFER && payment_mode == PAYMENT_MODE_MANUAL {
            let info = get_store_info(pool, s3).await?;
            Some(CheckoutPaymentInfo {
                bank_accounts: info.bank_accounts,
                ewallet_numbers: info.ewallet_numbers,
                qris_image_url: info.qris_image_url,
                total_amount: total,
            })
        } else {
            None
        };

    Ok(CheckoutResponse {
        order_id: saved.id,
        order_no: saved.order_no,
        fulfillment_method: order.fulfillment_method,
        payment_method: order.payment_method,
        total_amount: total,
        items: response_items,
        payment_info,
        snap_token: None,
        redirect_url: None,
        midtrans_order_id: None,
    })
}

async fn upload_image(
    s3: Option<&ObjectStorage>,
    prefix: &str,
    scope: &str,
    bytes: Vec<u8>,
    content_type: &str,
) -> AppResult<(String, String)> {
    let storage =
        s3.ok_or_else(|| AppError::Internal("object storage is not configured".to_string()))?;
    let extension = s3::image_extension(content_type).ok_or_else(|| {
        AppError::Validation("unsupported image type (use jpeg, png, or webp)".to_string())
    })?;
    if bytes.len() > s3::MAX_IMAGE_BYTES {
        return Err(AppError::Validation(
            "image is too large (max 2 MB)".to_string(),
        ));
    }

    let key = build_object_key(prefix, scope, extension);
    storage
        .put_object(&key, bytes, content_type)
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;
    let url = storage.public_url(&key);

    Ok((key, url))
}

/// Pengaturan toko yang tervalidasi, siap disimpan.
pub struct ValidatedStoreSettings {
    pub store_name: Option<String>,
    pub operating_hours: Option<String>,
    pub bank_accounts: Option<String>,
    pub ewallet_numbers: Option<String>,
    pub delivery_fee: Option<f64>,
    pub shop_whatsapp: Option<String>,
    pub payment_mode: Option<String>,
}

fn check_text_len(value: &str, field: &str, max: usize) -> Result<String, String> {
    let trimmed = value.trim().to_string();
    if trimmed.len() > max {
        return Err(format!("{field} is too long (max {max})"));
    }
    Ok(trimmed)
}

/// Validasi pembaruan pengaturan toko (murni, tanpa database).
pub fn validate_store_settings(
    request: &UpdateStoreSettingsRequest,
) -> Result<ValidatedStoreSettings, String> {
    let store_name = request
        .store_name
        .as_deref()
        .map(|v| {
            let trimmed = check_text_len(v, "store_name", 200)?;
            if trimmed.is_empty() {
                return Err("store_name must not be empty".to_string());
            }
            Ok(trimmed)
        })
        .transpose()?;
    let operating_hours = request
        .operating_hours
        .as_deref()
        .map(|v| check_text_len(v, "operating_hours", 500))
        .transpose()?;
    let bank_accounts = request
        .bank_accounts
        .as_deref()
        .map(|v| check_text_len(v, "bank_accounts", 2000))
        .transpose()?;
    let ewallet_numbers = request
        .ewallet_numbers
        .as_deref()
        .map(|v| check_text_len(v, "ewallet_numbers", 2000))
        .transpose()?;
    let delivery_fee = request
        .delivery_fee
        .map(|fee| {
            if !fee.is_finite() || fee < 0.0 || fee > 1_000_000_000_000.0 {
                return Err("delivery_fee must be between 0 and 1000000000000".to_string());
            }
            Ok(fee)
        })
        .transpose()?;
    let shop_whatsapp = request
        .shop_whatsapp
        .as_deref()
        .map(|v| {
            let trimmed = check_text_len(v, "shop_whatsapp", 30)?;
            if !trimmed.is_empty()
                && !trimmed
                    .chars()
                    .all(|c| c.is_ascii_digit() || "+- ().".contains(c))
            {
                return Err("shop_whatsapp contains invalid characters".to_string());
            }
            Ok(trimmed)
        })
        .transpose()?;
    let payment_mode = request
        .payment_mode
        .as_deref()
        .map(|v| match v.trim() {
            PAYMENT_MODE_MANUAL => Ok(PAYMENT_MODE_MANUAL.to_string()),
            PAYMENT_MODE_MIDTRANS => Ok(PAYMENT_MODE_MIDTRANS.to_string()),
            _ => Err("payment_mode must be manual or midtrans".to_string()),
        })
        .transpose()?;

    Ok(ValidatedStoreSettings {
        store_name,
        operating_hours,
        bank_accounts,
        ewallet_numbers,
        delivery_fee,
        shop_whatsapp,
        payment_mode,
    })
}

/// Baca pengaturan toko untuk dashboard kasir/admin (bentuk sama dengan info publik).
pub async fn get_store_settings(
    pool: &PgPool,
    s3: Option<&ObjectStorage>,
) -> AppResult<StoreInfoResponse> {
    get_store_info(pool, s3).await
}

/// Terapkan pembaruan pengaturan toko yang sudah tervalidasi.
pub async fn update_store_settings(
    pool: &PgPool,
    s3: Option<&ObjectStorage>,
    updated_by: i64,
    request: UpdateStoreSettingsRequest,
) -> AppResult<StoreInfoResponse> {
    let settings = validate_store_settings(&request).map_err(AppError::Validation)?;

    if let Some(value) = settings.store_name {
        repository::set_setting(pool, "store_name", &value, Some(updated_by)).await?;
    }
    if let Some(value) = settings.operating_hours {
        repository::set_setting(pool, "operating_hours", &value, Some(updated_by)).await?;
    }
    if let Some(value) = settings.bank_accounts {
        repository::set_setting(pool, "bank_accounts", &value, Some(updated_by)).await?;
    }
    if let Some(value) = settings.ewallet_numbers {
        repository::set_setting(pool, "ewallet_numbers", &value, Some(updated_by)).await?;
    }
    if let Some(fee) = settings.delivery_fee {
        repository::set_setting(pool, "delivery_fee", &fee.to_string(), Some(updated_by)).await?;
    }
    if let Some(value) = settings.shop_whatsapp {
        repository::set_setting(pool, "shop_whatsapp", &value, Some(updated_by)).await?;
    }
    if let Some(value) = settings.payment_mode {
        repository::set_setting(pool, "payment_mode", &value, Some(updated_by)).await?;
    }

    get_store_info(pool, s3).await
}

pub async fn upload_qris_image(
    pool: &PgPool,
    s3: Option<&ObjectStorage>,
    updated_by: i64,
    bytes: Vec<u8>,
    content_type: &str,
) -> AppResult<UploadResponse> {
    let (key, url) = upload_image(s3, "qris", "store", bytes, content_type).await?;
    repository::set_setting(pool, "qris_image_key", &key, Some(updated_by)).await?;

    Ok(UploadResponse { key, url })
}

// ---------------------------------------------------------------------------
// Dashboard kasir
// ---------------------------------------------------------------------------

/// Aksi kasir pada pesanan (satu tombol utama per status, Bagian 4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderAction {
    Pack,
    Ready,
    Deliver,
    Complete,
    Cancel,
    MarkPaid,
}

/// Aturan transisi status murni (Bagian 4.3 + keputusan A).
/// Mengembalikan status tujuan bila aksi sah untuk kondisi pesanan.
pub fn next_status(
    action: OrderAction,
    status: &str,
    fulfillment_method: &str,
    payment_method: &str,
    payment_status: &str,
) -> Result<&'static str, String> {
    match action {
        OrderAction::Pack => match status {
            "new" => Ok("packing"),
            _ => Err(format!(
                "only new orders can be packed (status is {status})"
            )),
        },
        OrderAction::Ready => match (status, fulfillment_method) {
            ("packing", "pickup") => Ok("ready"),
            ("packing", _) => Err("only pickup orders can be marked ready".to_string()),
            _ => Err(format!(
                "only packing orders can be marked ready (status is {status})"
            )),
        },
        OrderAction::Deliver => match (status, fulfillment_method) {
            ("packing", "delivery") if payment_method == "pay_at_store" => Ok("delivering"),
            ("packing", "delivery") if payment_status == "paid" => Ok("delivering"),
            ("packing", "delivery") => {
                Err("transfer payment must be confirmed before delivery".to_string())
            }
            ("packing", _) => Err("only delivery orders can be marked delivering".to_string()),
            _ => Err(format!(
                "only packing orders can be marked delivering (status is {status})"
            )),
        },
        OrderAction::Complete => match status {
            "ready" | "delivering" => Ok("completed"),
            _ => Err(format!(
                "only ready or delivering orders can be completed (status is {status})"
            )),
        },
        OrderAction::Cancel => match status {
            "new" | "packing" => Ok("cancelled"),
            _ => Err(format!(
                "orders with status {status} cannot be cancelled here"
            )),
        },
        OrderAction::MarkPaid => match (status, payment_status) {
            ("completed" | "cancelled", _) => Err(format!(
                "payment of {status} orders can no longer be changed"
            )),
            (_, "paid") => Err("payment is already confirmed".to_string()),
            _ => Ok("paid"),
        },
    }
}

fn to_response(order: OrderRow) -> OnlineOrderResponse {
    OnlineOrderResponse {
        id: order.id,
        order_no: order.order_no,
        customer_name: order.customer_name,
        customer_phone: order.customer_phone,
        fulfillment_method: order.fulfillment_method,
        delivery_address: order.delivery_address,
        payment_method: order.payment_method,
        payment_status: order.payment_status,
        status: order.status,
        subtotal_amount: order.subtotal_amount,
        delivery_fee: order.delivery_fee,
        total_amount: order.total_amount,
        notes: order.notes,
        sale_id: order.sale_id,
        cancel_reason: order.cancel_reason,
        member_id: order.member_id,
    }
}

pub async fn list_orders(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<(Vec<OnlineOrderResponse>, PaginationMeta)> {
    let (rows, meta) = repository::list_orders(pool, &query).await?;
    Ok((rows.into_iter().map(to_response).collect(), meta))
}

pub async fn find_order(pool: &PgPool, id: i64) -> AppResult<OnlineOrderDetailResponse> {
    let order = repository::find_order(pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    let items = repository::order_items(pool, id).await?;

    Ok(OnlineOrderDetailResponse {
        order: to_response(order),
        items: items
            .into_iter()
            .map(|item| OnlineOrderItemResponse {
                id: item.id,
                product_sku_id: item.product_sku_id,
                product_name: item.product_name,
                product_sku: item.product_sku,
                unit: item.unit,
                quantity: item.quantity,
                unit_price: item.unit_price,
                line_total: item.line_total,
            })
            .collect(),
    })
}

async fn transition(
    pool: &PgPool,
    user_id: i64,
    id: i64,
    action: OrderAction,
    audit_action: &str,
) -> AppResult<OnlineOrderDetailResponse> {
    let mut tx = pool.begin().await?;
    let order = repository::lock_order(&mut tx, id)
        .await?
        .ok_or(AppError::NotFound)?;
    let target = next_status(
        action,
        &order.status,
        &order.fulfillment_method,
        &order.payment_method,
        &order.payment_status,
    )
    .map_err(AppError::Validation)?;
    repository::set_order_status(&mut tx, id, target, user_id).await?;
    tx.commit().await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(Some(user_id), audit_action, "online_order", Some(id)),
    )
    .await;

    find_order(pool, id).await
}

pub async fn pack_order(
    pool: &PgPool,
    user_id: i64,
    id: i64,
) -> AppResult<OnlineOrderDetailResponse> {
    let mut tx = pool.begin().await?;
    let order = repository::lock_order(&mut tx, id)
        .await?
        .ok_or(AppError::NotFound)?;
    next_status(
        OrderAction::Pack,
        &order.status,
        &order.fulfillment_method,
        &order.payment_method,
        &order.payment_status,
    )
    .map_err(AppError::Validation)?;

    let items = repository::order_items(pool, id).await?;
    for item in &items {
        repository::cut_stock_for_pack(&mut tx, item, user_id, id).await?;
    }
    repository::set_order_status(&mut tx, id, "packing", user_id).await?;
    tx.commit().await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(user_id),
            "online_order.pack",
            "online_order",
            Some(id),
        ),
    )
    .await;

    find_order(pool, id).await
}

pub async fn ready_order(
    pool: &PgPool,
    user_id: i64,
    id: i64,
) -> AppResult<OnlineOrderDetailResponse> {
    transition(pool, user_id, id, OrderAction::Ready, "online_order.ready").await
}

pub async fn deliver_order(
    pool: &PgPool,
    user_id: i64,
    id: i64,
) -> AppResult<OnlineOrderDetailResponse> {
    transition(
        pool,
        user_id,
        id,
        OrderAction::Deliver,
        "online_order.deliver",
    )
    .await
}

pub async fn complete_order(
    pool: &PgPool,
    user_id: i64,
    id: i64,
) -> AppResult<OnlineOrderDetailResponse> {
    let mut tx = pool.begin().await?;
    let order = repository::lock_order(&mut tx, id)
        .await?
        .ok_or(AppError::NotFound)?;
    next_status(
        OrderAction::Complete,
        &order.status,
        &order.fulfillment_method,
        &order.payment_method,
        &order.payment_status,
    )
    .map_err(AppError::Validation)?;

    let session_id = repository::find_cashier_open_session(&mut tx, user_id)
        .await?
        .ok_or_else(|| {
            AppError::Conflict(
                "open a cash register session before completing online orders".to_string(),
            )
        })?;

    let items = repository::order_items(pool, id).await?;
    let (sale_id, links) =
        repository::insert_online_sale(&mut tx, &order, &items, user_id, session_id).await?;
    for (order_item_id, sale_item_id) in links {
        repository::link_movements_to_sale(&mut tx, order_item_id, sale_id, sale_item_id).await?;
    }
    repository::finalize_completed_order(&mut tx, id, sale_id, user_id).await?;
    tx.commit().await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(user_id),
            "online_order.complete",
            "online_order",
            Some(id),
        )
        .with_metadata(json!({ "sale_id": sale_id })),
    )
    .await;

    find_order(pool, id).await
}

pub async fn cancel_order(
    pool: &PgPool,
    user_id: i64,
    id: i64,
    reason: Option<String>,
) -> AppResult<OnlineOrderDetailResponse> {
    let mut tx = pool.begin().await?;
    let order = repository::lock_order(&mut tx, id)
        .await?
        .ok_or(AppError::NotFound)?;
    next_status(
        OrderAction::Cancel,
        &order.status,
        &order.fulfillment_method,
        &order.payment_method,
        &order.payment_status,
    )
    .map_err(AppError::Validation)?;

    if order.status == "packing" {
        let items = repository::order_items(pool, id).await?;
        for item in &items {
            repository::restore_stock_for_cancel(&mut tx, item, user_id, id).await?;
        }
    }
    let reason = reason
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty());
    repository::set_order_cancelled(&mut tx, id, reason.as_deref(), user_id).await?;
    tx.commit().await?;

    audit::record_audit_log_best_effort(
        pool,
        audit::AuditLogRecord::success(
            Some(user_id),
            "online_order.cancel",
            "online_order",
            Some(id),
        )
        .with_metadata(json!({ "reason": reason })),
    )
    .await;

    find_order(pool, id).await
}

pub async fn mark_order_paid(
    pool: &PgPool,
    user_id: i64,
    id: i64,
) -> AppResult<OnlineOrderDetailResponse> {
    let mut tx = pool.begin().await?;
    let order = repository::lock_order(&mut tx, id)
        .await?
        .ok_or(AppError::NotFound)?;
    next_status(
        OrderAction::MarkPaid,
        &order.status,
        &order.fulfillment_method,
        &order.payment_method,
        &order.payment_status,
    )
    .map_err(AppError::Validation)?;
    repository::set_order_paid(&mut tx, id, user_id).await?;
    tx.commit().await?;

    find_order(pool, id).await
}

#[cfg(test)]
mod tests {
    use super::super::dto::CheckoutItemRequest;
    use super::*;

    fn checkout_request() -> CheckoutRequest {
        CheckoutRequest {
            customer_name: "Budi".to_string(),
            customer_phone: "08123456789".to_string(),
            fulfillment_method: FULFILLMENT_PICKUP.to_string(),
            delivery_address: None,
            payment_method: PAYMENT_PAY_AT_STORE.to_string(),
            notes: None,
            items: vec![CheckoutItemRequest {
                product_sku_id: 1,
                quantity: 2.0,
            }],
        }
    }

    fn member_profile() -> MemberResponse {
        MemberResponse {
            id: 9,
            member_no: "MBR-20261109-000001".to_string(),
            full_name: "Siti Member".to_string(),
            phone: "0899000111222".to_string(),
            email: None,
            address: Some("Jl. Anggrek 5".to_string()),
            joined_at: chrono::Utc::now(),
            is_active: true,
            notes: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn resolve_identity_prefers_member_profile_over_body() {
        // Anti-spoof: nama/telepon body (guest) diabaikan saat member login.
        let mut request = checkout_request();
        request.customer_name = "Penyusup".to_string();
        request.customer_phone = "080000000000".to_string();

        let identity = resolve_checkout_identity(&request, Some(&member_profile()));

        assert_eq!(identity.customer_name, "Siti Member");
        assert_eq!(identity.customer_phone, "0899000111222");
        assert_eq!(identity.member_id, Some(9));
    }

    #[test]
    fn resolve_identity_defaults_address_from_profile_but_body_wins() {
        let member = member_profile();

        let request = checkout_request();
        let identity = resolve_checkout_identity(&request, Some(&member));
        assert_eq!(identity.delivery_address.as_deref(), Some("Jl. Anggrek 5"));

        let mut request = checkout_request();
        request.delivery_address = Some("  Kantor Cabang 2  ".to_string());
        let identity = resolve_checkout_identity(&request, Some(&member));
        assert_eq!(
            identity.delivery_address.as_deref(),
            Some("Kantor Cabang 2")
        );
    }

    #[test]
    fn resolve_identity_guest_uses_body_without_attribution() {
        let request = checkout_request();
        let identity = resolve_checkout_identity(&request, None);

        assert_eq!(identity.customer_name, "Budi");
        assert_eq!(identity.customer_phone, "08123456789");
        assert_eq!(identity.member_id, None);
    }

    #[test]
    fn resolve_identity_member_checkout_valid_despite_empty_body_identity() {
        // Body boleh kosong untuk member: identitas efektif dari profil.
        let mut request = checkout_request();
        request.customer_name = String::new();
        request.customer_phone = "   ".to_string();

        let identity = resolve_checkout_identity(&request, Some(&member_profile()));
        let effective = CheckoutRequest {
            customer_name: identity.customer_name,
            customer_phone: identity.customer_phone,
            fulfillment_method: request.fulfillment_method,
            delivery_address: identity.delivery_address,
            payment_method: request.payment_method,
            notes: request.notes,
            items: request.items,
        };

        assert!(validate_checkout(&effective, PAYMENT_MODE_MANUAL).is_ok());
    }

    #[test]
    fn validate_checkout_accepts_pickup_order() {
        let order =
            validate_checkout(&checkout_request(), PAYMENT_MODE_MANUAL).expect("valid order");
        assert_eq!(order.customer_name, "Budi");
        assert_eq!(order.items.len(), 1);
        assert!(order.delivery_address.is_none());
    }

    #[test]
    fn validate_checkout_rejects_missing_identity() {
        let mut request = checkout_request();
        request.customer_name = "   ".to_string();
        assert!(validate_checkout(&request, PAYMENT_MODE_MANUAL).is_err());

        let mut request = checkout_request();
        request.customer_phone = String::new();
        assert!(validate_checkout(&request, PAYMENT_MODE_MANUAL).is_err());
    }

    #[test]
    fn validate_checkout_requires_address_for_delivery() {
        let mut request = checkout_request();
        request.fulfillment_method = FULFILLMENT_DELIVERY.to_string();
        assert!(validate_checkout(&request, PAYMENT_MODE_MANUAL).is_err());

        request.delivery_address = Some("Jl. Mawar 1".to_string());
        assert!(validate_checkout(&request, PAYMENT_MODE_MANUAL).is_ok());
    }

    #[test]
    fn validate_checkout_rejects_bad_enums_and_items() {
        let mut request = checkout_request();
        request.fulfillment_method = "drone".to_string();
        assert!(validate_checkout(&request, PAYMENT_MODE_MANUAL).is_err());

        let mut request = checkout_request();
        request.payment_method = "crypto".to_string();
        assert!(validate_checkout(&request, PAYMENT_MODE_MANUAL).is_err());
        assert!(validate_checkout(&request, PAYMENT_MODE_MIDTRANS).is_err());

        let mut request = checkout_request();
        request.items.clear();
        assert!(validate_checkout(&request, PAYMENT_MODE_MANUAL).is_err());

        let mut request = checkout_request();
        request.items[0].quantity = 0.0;
        assert!(validate_checkout(&request, PAYMENT_MODE_MANUAL).is_err());
    }

    #[test]
    fn normalize_payment_mode_defaults_to_manual() {
        assert_eq!(normalize_payment_mode(None), PAYMENT_MODE_MANUAL);
        assert_eq!(normalize_payment_mode(Some("")), PAYMENT_MODE_MANUAL);
        assert_eq!(
            normalize_payment_mode(Some("transfer")),
            PAYMENT_MODE_MANUAL
        );
        assert_eq!(
            normalize_payment_mode(Some(" manual ")),
            PAYMENT_MODE_MANUAL
        );
        assert_eq!(
            normalize_payment_mode(Some("midtrans")),
            PAYMENT_MODE_MIDTRANS
        );
        assert_eq!(
            normalize_payment_mode(Some("  midtrans  ")),
            PAYMENT_MODE_MIDTRANS
        );
    }

    #[test]
    fn validate_checkout_manual_mode_accepts_cash_and_transfer() {
        let mut request = checkout_request();
        request.payment_method = PAYMENT_PAY_AT_STORE.to_string();
        assert!(validate_checkout(&request, PAYMENT_MODE_MANUAL).is_ok());

        let mut request = checkout_request();
        request.payment_method = PAYMENT_TRANSFER.to_string();
        let order = validate_checkout(&request, PAYMENT_MODE_MANUAL).expect("transfer valid");
        assert_eq!(order.payment_method, PAYMENT_TRANSFER);
    }

    #[test]
    fn validate_checkout_manual_mode_rejects_midtrans() {
        let mut request = checkout_request();
        request.payment_method = PAYMENT_MIDTRANS.to_string();
        let error = validate_checkout(&request, PAYMENT_MODE_MANUAL).expect_err("must reject");
        assert!(error.contains("manual payment mode"), "{error}");
    }

    #[test]
    fn validate_checkout_midtrans_mode_accepts_cash_and_midtrans() {
        let mut request = checkout_request();
        request.payment_method = PAYMENT_PAY_AT_STORE.to_string();
        assert!(validate_checkout(&request, PAYMENT_MODE_MIDTRANS).is_ok());

        let mut request = checkout_request();
        request.payment_method = PAYMENT_MIDTRANS.to_string();
        let order = validate_checkout(&request, PAYMENT_MODE_MIDTRANS).expect("midtrans valid");
        assert_eq!(order.payment_method, PAYMENT_MIDTRANS);
    }

    #[test]
    fn validate_checkout_midtrans_mode_rejects_transfer() {
        let mut request = checkout_request();
        request.payment_method = PAYMENT_TRANSFER.to_string();
        let error = validate_checkout(&request, PAYMENT_MODE_MIDTRANS).expect_err("must reject");
        assert!(error.contains("midtrans payment mode"), "{error}");
    }

    #[test]
    fn validate_checkout_unknown_mode_behaves_like_manual() {
        let mut request = checkout_request();
        request.payment_method = PAYMENT_TRANSFER.to_string();
        assert!(validate_checkout(&request, "unknown").is_ok());

        let mut request = checkout_request();
        request.payment_method = PAYMENT_MIDTRANS.to_string();
        assert!(validate_checkout(&request, "unknown").is_err());
    }

    #[test]
    fn normalize_whatsapp_converts_local_format() {
        assert_eq!(normalize_whatsapp("0812-3456-789"), "628123456789");
        assert_eq!(normalize_whatsapp("+62 812 3456 789"), "628123456789");
        assert_eq!(normalize_whatsapp("628123456789"), "628123456789");
        assert_eq!(normalize_whatsapp(""), "");
    }

    #[test]
    fn build_object_key_uses_prefix_scope_and_extension() {
        let key = build_object_key("products", "42", "jpg");
        assert!(key.starts_with("products/42/"));
        assert!(key.ends_with(".jpg"));
        assert!(key.len() > "products/42/.jpg".len());
    }

    use super::OrderAction;

    #[test]
    fn transition_follows_documented_lifecycle() {
        // new -> packing -> ready -> completed (pickup)
        assert_eq!(
            next_status(OrderAction::Pack, "new", "pickup", "pay_at_store", "unpaid"),
            Ok("packing")
        );
        assert_eq!(
            next_status(
                OrderAction::Ready,
                "packing",
                "pickup",
                "pay_at_store",
                "unpaid"
            ),
            Ok("ready")
        );
        assert_eq!(
            next_status(
                OrderAction::Complete,
                "ready",
                "pickup",
                "pay_at_store",
                "paid"
            ),
            Ok("completed")
        );
        // packing -> delivering -> completed (delivery + transfer paid)
        assert_eq!(
            next_status(
                OrderAction::Deliver,
                "packing",
                "delivery",
                "transfer",
                "paid"
            ),
            Ok("delivering")
        );
        assert_eq!(
            next_status(
                OrderAction::Deliver,
                "packing",
                "delivery",
                "pay_at_store",
                "unpaid"
            ),
            Ok("delivering")
        );
        assert_eq!(
            next_status(
                OrderAction::Complete,
                "delivering",
                "delivery",
                "transfer",
                "paid"
            ),
            Ok("completed")
        );
    }

    #[test]
    fn transition_rejects_wrong_fulfillment_and_unpaid_delivery() {
        // ready hanya untuk pickup
        assert!(
            next_status(
                OrderAction::Ready,
                "packing",
                "delivery",
                "transfer",
                "paid"
            )
            .is_err()
        );
        // deliver hanya untuk antar
        assert!(
            next_status(
                OrderAction::Deliver,
                "packing",
                "pickup",
                "pay_at_store",
                "unpaid"
            )
            .is_err()
        );
        // antar + transfer wajib paid sebelum delivering
        assert!(
            next_status(
                OrderAction::Deliver,
                "packing",
                "delivery",
                "transfer",
                "unpaid"
            )
            .is_err()
        );
        // complete hanya dari ready/delivering
        assert!(
            next_status(
                OrderAction::Complete,
                "new",
                "pickup",
                "pay_at_store",
                "unpaid"
            )
            .is_err()
        );
        assert!(
            next_status(
                OrderAction::Complete,
                "packing",
                "pickup",
                "pay_at_store",
                "unpaid"
            )
            .is_err()
        );
        // pack ulang ditolak
        assert!(
            next_status(
                OrderAction::Pack,
                "packing",
                "pickup",
                "pay_at_store",
                "unpaid"
            )
            .is_err()
        );
    }

    #[test]
    fn validate_store_settings_accepts_partial_update() {
        use super::super::dto::UpdateStoreSettingsRequest;
        let request = UpdateStoreSettingsRequest {
            store_name: Some("  Toko Maju  ".to_string()),
            operating_hours: None,
            bank_accounts: Some(String::new()),
            ewallet_numbers: None,
            delivery_fee: Some(5000.0),
            shop_whatsapp: Some("0812-3456-789".to_string()),
            payment_mode: Some("midtrans".to_string()),
        };
        let settings = validate_store_settings(&request).expect("valid settings");
        assert_eq!(settings.store_name.as_deref(), Some("Toko Maju"));
        assert!(settings.operating_hours.is_none());
        assert_eq!(settings.bank_accounts.as_deref(), Some(""));
        assert_eq!(settings.delivery_fee, Some(5000.0));
        assert_eq!(settings.shop_whatsapp.as_deref(), Some("0812-3456-789"));
        assert_eq!(settings.payment_mode.as_deref(), Some("midtrans"));
    }

    #[test]
    fn validate_store_settings_rejects_bad_values() {
        use super::super::dto::UpdateStoreSettingsRequest;
        let base = || UpdateStoreSettingsRequest {
            store_name: None,
            operating_hours: None,
            bank_accounts: None,
            ewallet_numbers: None,
            delivery_fee: None,
            shop_whatsapp: None,
            payment_mode: None,
        };

        let mut request = base();
        request.store_name = Some("   ".to_string());
        assert!(validate_store_settings(&request).is_err());

        let mut request = base();
        request.delivery_fee = Some(-100.0);
        assert!(validate_store_settings(&request).is_err());

        let mut request = base();
        request.delivery_fee = Some(f64::NAN);
        assert!(validate_store_settings(&request).is_err());

        let mut request = base();
        request.shop_whatsapp = Some("abc-def".to_string());
        assert!(validate_store_settings(&request).is_err());

        let mut request = base();
        request.shop_whatsapp = Some("08123456789".to_string());
        assert!(validate_store_settings(&request).is_ok());

        let mut request = base();
        request.payment_mode = Some("manual".to_string());
        assert!(validate_store_settings(&request).is_ok());

        let mut request = base();
        request.payment_mode = Some("midtrans".to_string());
        assert!(validate_store_settings(&request).is_ok());

        let mut request = base();
        request.payment_mode = Some("transfer".to_string());
        assert!(validate_store_settings(&request).is_err());

        let mut request = base();
        request.payment_mode = Some("  ".to_string());
        assert!(validate_store_settings(&request).is_err());
    }

    #[test]
    fn cancel_and_mark_paid_follow_rules() {
        assert_eq!(
            next_status(
                OrderAction::Cancel,
                "new",
                "pickup",
                "pay_at_store",
                "unpaid"
            ),
            Ok("cancelled")
        );
        assert_eq!(
            next_status(
                OrderAction::Cancel,
                "packing",
                "delivery",
                "transfer",
                "unpaid"
            ),
            Ok("cancelled")
        );
        assert!(
            next_status(
                OrderAction::Cancel,
                "ready",
                "pickup",
                "pay_at_store",
                "unpaid"
            )
            .is_err()
        );
        assert!(
            next_status(
                OrderAction::Cancel,
                "delivering",
                "delivery",
                "transfer",
                "paid"
            )
            .is_err()
        );
        assert!(
            next_status(
                OrderAction::Cancel,
                "completed",
                "pickup",
                "pay_at_store",
                "paid"
            )
            .is_err()
        );
        // mark-paid boleh selama belum selesai/batal dan belum paid
        assert_eq!(
            next_status(
                OrderAction::MarkPaid,
                "new",
                "delivery",
                "transfer",
                "unpaid"
            ),
            Ok("paid")
        );
        assert!(
            next_status(
                OrderAction::MarkPaid,
                "packing",
                "delivery",
                "transfer",
                "paid"
            )
            .is_err()
        );
        assert!(
            next_status(
                OrderAction::MarkPaid,
                "completed",
                "pickup",
                "pay_at_store",
                "paid"
            )
            .is_err()
        );
        assert!(
            next_status(
                OrderAction::MarkPaid,
                "cancelled",
                "pickup",
                "pay_at_store",
                "unpaid"
            )
            .is_err()
        );
    }
}
