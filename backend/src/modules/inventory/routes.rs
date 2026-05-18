use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/stock-adjustments",
            get(handler::list_stock_adjustments).post(handler::create_stock_adjustment),
        )
        .route(
            "/stock-adjustments/{id}",
            get(handler::get_stock_adjustment),
        )
        .route(
            "/stock-adjustments/{id}/post",
            post(handler::post_stock_adjustment),
        )
        .route(
            "/stock-adjustments/{id}/cancel",
            post(handler::cancel_stock_adjustment),
        )
}
