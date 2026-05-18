use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/sales",
            get(handler::list_sales).post(handler::create_sale),
        )
        .route("/sales/{id}", get(handler::get_sale))
        .route("/sales/{id}/cancel", post(handler::cancel_sale))
}
