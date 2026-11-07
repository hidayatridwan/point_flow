use std::time::Duration;

use axum::{
    Json, Router,
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use tower_governor::{GovernorError, GovernorLayer, governor::GovernorConfigBuilder};

use crate::{app::AppState, models::ApiResponse};

use super::handler;

/// 429 dari governor diubah ke envelope error repo agar konsisten.
fn rate_limit_response(_error: GovernorError) -> Response<Body> {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(ApiResponse::<()>::error(
            "rate_limited",
            "Rate limit exceeded, please try again later.".to_string(),
        )),
    )
        .into_response()
}

/// Route publik: tanpa `AuthUser`, hanya rate limit + envelope 429.
/// Kuota baca (katalog, kategori, info toko): 60/menit per IP, burst 60.
/// Kuota tulis (checkout): 10/menit per IP, burst 10. Tuning saat implementasi.
pub fn public_routes() -> Router<AppState> {
    // Gagal di sini berarti konstanta kuota tidak valid; panic saat boot.
    let catalog_config = GovernorConfigBuilder::default()
        .period(Duration::from_secs(60))
        .burst_size(60)
        .finish()
        .expect("catalog rate limit config is valid");
    let checkout_config = GovernorConfigBuilder::default()
        .period(Duration::from_secs(60))
        .burst_size(10)
        .finish()
        .expect("checkout rate limit config is valid");

    Router::new()
        .route("/public/products", get(handler::list_public_products))
        .route("/public/categories", get(handler::list_public_categories))
        .route("/public/store-info", get(handler::get_public_store_info))
        .layer(GovernorLayer::new(catalog_config).error_handler(rate_limit_response))
        .merge(
            Router::new()
                .route("/public/orders", post(handler::checkout))
                .layer(GovernorLayer::new(checkout_config).error_handler(rate_limit_response)),
        )
}

/// Route kasir/admin: JWT + permission `online_orders`.
pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/store-settings/qris-image",
        post(handler::upload_qris_image),
    )
}
