use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/sale-returns",
            get(handler::list_sale_returns).post(handler::create_sale_return),
        )
        .route("/sale-returns/{id}", get(handler::get_sale_return))
        .route(
            "/sale-returns/{id}/cancel",
            post(handler::cancel_sale_return),
        )
        .route(
            "/sales/{id}/returnable-items",
            get(handler::list_returnable_sale_items),
        )
}
