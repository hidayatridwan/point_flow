use serde_json::Value;
use sqlx::{PgPool, Postgres, Transaction};

use crate::error::AppResult;

use super::model::{ChargeItem, MidtransPayment};

const PAYMENT_SELECT: &str = r#"
    SELECT
        id,
        order_id,
        source_type,
        gross_amount,
        snap_token,
        redirect_url,
        transaction_status,
        fraud_status,
        payment_type,
        va_info,
        raw_notification,
        sale_id,
        online_order_id,
        items,
        member_id,
        notes,
        expires_at,
        settled_at,
        created_by,
        created_at,
        updated_at
    FROM midtrans_payments
"#;

#[allow(clippy::too_many_arguments)]
pub async fn insert_payment(
    tx: &mut Transaction<'_, Postgres>,
    order_id: &str,
    source_type: &str,
    gross_amount: i64,
    snap_token: Option<&str>,
    redirect_url: Option<&str>,
    items: &Value,
    member_id: Option<i64>,
    notes: Option<&str>,
    online_order_id: Option<i64>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    // Kasir pembuat charge; `None` untuk checkout publik online (tanpa kasir).
    created_by: Option<i64>,
) -> AppResult<MidtransPayment> {
    // INSERT ... RETURNING memakai kolom yang sama dengan SELECT di atas.
    sqlx::query_as::<_, MidtransPayment>(
        r#"
        INSERT INTO midtrans_payments (
            order_id, source_type, gross_amount, snap_token, redirect_url,
            items, member_id, notes, online_order_id, expires_at, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING
            id, order_id, source_type, gross_amount, snap_token, redirect_url,
            transaction_status, fraud_status, payment_type, va_info,
            raw_notification, sale_id, online_order_id, items, member_id,
            notes, expires_at, settled_at, created_by, created_at, updated_at
        "#,
    )
    .bind(order_id)
    .bind(source_type)
    .bind(gross_amount as i32)
    .bind(snap_token)
    .bind(redirect_url)
    .bind(items)
    .bind(member_id)
    .bind(notes)
    .bind(online_order_id)
    .bind(expires_at)
    .bind(created_by)
    .fetch_one(&mut **tx)
    .await
    .map_err(Into::into)
}

pub async fn find_by_order_id(pool: &PgPool, order_id: &str) -> AppResult<Option<MidtransPayment>> {
    sqlx::query_as::<_, MidtransPayment>(&format!("{PAYMENT_SELECT} WHERE order_id = $1"))
        .bind(order_id)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

pub async fn lock_by_order_id(
    tx: &mut Transaction<'_, Postgres>,
    order_id: &str,
) -> AppResult<Option<MidtransPayment>> {
    sqlx::query_as::<_, MidtransPayment>(&format!(
        "{PAYMENT_SELECT} WHERE order_id = $1 FOR UPDATE"
    ))
    .bind(order_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(Into::into)
}

/// Update status + info pembayaran dari notifikasi/status Midtrans.
#[allow(clippy::too_many_arguments)]
pub async fn update_status(
    tx: &mut Transaction<'_, Postgres>,
    order_id: &str,
    transaction_status: &str,
    fraud_status: Option<&str>,
    payment_type: Option<&str>,
    va_info: Option<&Value>,
    raw_notification: &Value,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE midtrans_payments
        SET transaction_status = $2,
            fraud_status = $3,
            payment_type = $4,
            va_info = COALESCE($5, va_info),
            raw_notification = $6,
            updated_at = now()
        WHERE order_id = $1
        "#,
    )
    .bind(order_id)
    .bind(transaction_status)
    .bind(fraud_status)
    .bind(payment_type)
    .bind(va_info)
    .bind(raw_notification)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Tandai lunas: kunci `settled_at` (guard idempotensi) + tautan sale/order.
pub async fn mark_settled(
    tx: &mut Transaction<'_, Postgres>,
    order_id: &str,
    sale_id: Option<i64>,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"
        UPDATE midtrans_payments
        SET settled_at = now(), sale_id = COALESCE($2, sale_id), updated_at = now()
        WHERE order_id = $1 AND settled_at IS NULL
        "#,
    )
    .bind(order_id)
    .bind(sale_id)
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected() == 1)
}

/// Sesi register yang sedang terbuka saat settlement terjadi (sesi baru,
/// bukan sesi asal charge yang mungkin sudah tutup). Terbaru didahulukan.
pub async fn find_open_session(tx: &mut Transaction<'_, Postgres>) -> AppResult<Option<i64>> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT crs.id
        FROM cash_register_sessions crs
        JOIN cash_registers cr ON cr.id = crs.cash_register_id
        WHERE crs.status = 'open'
            AND cr.status = 'active'
            AND cr.deleted_at IS NULL
        ORDER BY crs.id DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(Into::into)
}

pub async fn order_exists(pool: &PgPool, online_order_id: i64) -> AppResult<bool> {
    let id = sqlx::query_scalar::<_, i64>("SELECT id FROM online_orders WHERE id = $1")
        .bind(online_order_id)
        .fetch_optional(pool)
        .await?;

    Ok(id.is_some())
}

/// Mode pembayaran toko aktif (`manual` | `midtrans`). Nilai tak dikenal
/// (termasuk baris belum ada) dinormalkan ke `manual` agar toko yang sudah
/// jalan tidak berubah perilaku — sama seperti sisi online order.
pub async fn get_payment_mode(pool: &PgPool) -> AppResult<String> {
    let mode: Option<String> =
        sqlx::query_scalar("SELECT value FROM store_settings WHERE key = 'payment_mode'")
            .fetch_optional(pool)
            .await?;
    Ok(match mode.as_deref().map(str::trim) {
        Some("midtrans") => "midtrans".to_string(),
        _ => "manual".to_string(),
    })
}

pub fn stored_items(payment: &MidtransPayment) -> Vec<ChargeItem> {
    payment
        .items
        .as_ref()
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default()
}
