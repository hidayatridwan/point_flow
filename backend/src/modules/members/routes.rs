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

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/members",
            get(handler::list_members).post(handler::create_member),
        )
        .route(
            "/members/{id}",
            get(handler::get_member)
                .put(handler::update_member)
                .delete(handler::delete_member),
        )
        .route(
            "/members/{id}/reset-password",
            post(handler::reset_member_password),
        )
}

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

/// Route auth member publik (storefront): tanpa `AuthUser`, identitas
/// dibuktikan via `AuthMember` untuk endpoint privat. Rate limit mengikuti
/// pola ketat checkout online (10/menit per IP, burst 10) karena endpoint
/// login rawan brute-force.
pub fn public_routes() -> Router<AppState> {
    let auth_config = GovernorConfigBuilder::default()
        .period(Duration::from_secs(60))
        .burst_size(10)
        .finish()
        .expect("member auth rate limit config is valid");

    Router::new()
        .route("/members/auth/login", post(handler::member_login))
        .route("/members/auth/refresh", post(handler::member_refresh))
        .route("/members/auth/logout", post(handler::member_logout))
        .route("/members/auth/me", get(handler::member_me))
        .route(
            "/members/auth/change-password",
            post(handler::member_change_password),
        )
        .layer(GovernorLayer::new(auth_config).error_handler(rate_limit_response))
}
