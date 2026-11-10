use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/payments/midtrans/charges", post(handler::create_charge))
        .route("/payments/midtrans/{order_id}", get(handler::get_payment))
}

pub fn public_routes() -> Router<AppState> {
    Router::new().route(
        "/payments/midtrans/notification",
        post(handler::handle_notification),
    )
}
