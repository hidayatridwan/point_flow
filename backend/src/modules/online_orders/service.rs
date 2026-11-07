use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    infra::s3::{self, ObjectStorage},
    models::{PaginationMeta, UploadResponse},
};

use super::{
    dto::{
        CheckoutItemResponse, CheckoutPaymentInfo, CheckoutRequest, CheckoutResponse,
        PublicCatalogQuery, PublicCategory, PublicProductItem, StoreInfoResponse,
    },
    repository::{self, NewOrderItem},
};

pub const FULFILLMENT_PICKUP: &str = "pickup";
pub const FULFILLMENT_DELIVERY: &str = "delivery";
pub const PAYMENT_PAY_AT_STORE: &str = "pay_at_store";
pub const PAYMENT_TRANSFER: &str = "transfer";

const MAX_CHECKOUT_ITEMS: usize = 100;

pub struct ValidatedItem {
    pub product_sku_id: i64,
    pub quantity: f64,
}

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
pub fn validate_checkout(request: &CheckoutRequest) -> Result<ValidatedCheckout, String> {
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
    if payment_method != PAYMENT_PAY_AT_STORE && payment_method != PAYMENT_TRANSFER {
        return Err("payment_method must be pay_at_store or transfer".to_string());
    }

    let delivery_address = request.delivery_address.as_deref().map(str::trim).and_then(|s| {
        if s.is_empty() { None } else { Some(s.to_string()) }
    });
    if fulfillment_method == FULFILLMENT_DELIVERY && delivery_address.is_none() {
        return Err("delivery_address is required for delivery orders".to_string());
    }

    if request.items.is_empty() {
        return Err("items must not be empty".to_string());
    }
    if request.items.len() > MAX_CHECKOUT_ITEMS {
        return Err(format!(
            "items must not exceed {MAX_CHECKOUT_ITEMS}"
        ));
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
        if s.is_empty() { None } else { Some(s.to_string()) }
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

    Ok(StoreInfoResponse {
        store_name,
        operating_hours,
        bank_accounts,
        ewallet_numbers,
        qris_image_url,
        delivery_fee,
    })
}

pub async fn checkout(
    pool: &PgPool,
    s3: Option<&ObjectStorage>,
    request: CheckoutRequest,
) -> AppResult<CheckoutResponse> {
    let order = validate_checkout(&request).map_err(AppError::Validation)?;

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
            items,
        },
    )
    .await?;

    let payment_info = if order.payment_method == PAYMENT_TRANSFER {
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
    })
}

async fn upload_image(
    s3: Option<&ObjectStorage>,
    prefix: &str,
    scope: &str,
    bytes: Vec<u8>,
    content_type: &str,
) -> AppResult<(String, String)> {
    let storage = s3.ok_or_else(|| AppError::Internal("object storage is not configured".to_string()))?;
    let extension =
        s3::image_extension(content_type).ok_or_else(|| {
            AppError::Validation(
                "unsupported image type (use jpeg, png, or webp)".to_string(),
            )
        })?;
    if bytes.len() > s3::MAX_IMAGE_BYTES {
        return Err(AppError::Validation("image is too large (max 2 MB)".to_string()));
    }

    let key = build_object_key(prefix, scope, extension);
    storage
        .put_object(&key, bytes, content_type)
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;
    let url = storage.public_url(&key);

    Ok((key, url))
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

    #[test]
    fn validate_checkout_accepts_pickup_order() {
        let order = validate_checkout(&checkout_request()).expect("valid order");
        assert_eq!(order.customer_name, "Budi");
        assert_eq!(order.items.len(), 1);
        assert!(order.delivery_address.is_none());
    }

    #[test]
    fn validate_checkout_rejects_missing_identity() {
        let mut request = checkout_request();
        request.customer_name = "   ".to_string();
        assert!(validate_checkout(&request).is_err());

        let mut request = checkout_request();
        request.customer_phone = String::new();
        assert!(validate_checkout(&request).is_err());
    }

    #[test]
    fn validate_checkout_requires_address_for_delivery() {
        let mut request = checkout_request();
        request.fulfillment_method = FULFILLMENT_DELIVERY.to_string();
        assert!(validate_checkout(&request).is_err());

        request.delivery_address = Some("Jl. Mawar 1".to_string());
        assert!(validate_checkout(&request).is_ok());
    }

    #[test]
    fn validate_checkout_rejects_bad_enums_and_items() {
        let mut request = checkout_request();
        request.fulfillment_method = "drone".to_string();
        assert!(validate_checkout(&request).is_err());

        let mut request = checkout_request();
        request.payment_method = "crypto".to_string();
        assert!(validate_checkout(&request).is_err());

        let mut request = checkout_request();
        request.items.clear();
        assert!(validate_checkout(&request).is_err());

        let mut request = checkout_request();
        request.items[0].quantity = 0.0;
        assert!(validate_checkout(&request).is_err());
    }

    #[test]
    fn build_object_key_uses_prefix_scope_and_extension() {
        let key = build_object_key("products", "42", "jpg");
        assert!(key.starts_with("products/42/"));
        assert!(key.ends_with(".jpg"));
        assert!(key.len() > "products/42/.jpg".len());
    }
}
