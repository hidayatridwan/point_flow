use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/profit-sharing/periods",
            get(handler::list_periods).post(handler::create_period),
        )
        .route(
            "/profit-sharing/periods/{id}",
            get(handler::get_period).put(handler::update_period),
        )
        .route(
            "/profit-sharing/periods/{id}/calculate",
            post(handler::calculate_period),
        )
        .route(
            "/profit-sharing/periods/{id}/finalize",
            post(handler::finalize_period),
        )
        .route(
            "/profit-sharing/periods/{id}/cancel",
            post(handler::cancel_period),
        )
        .route(
            "/profit-sharing/periods/{id}/distributions",
            get(handler::list_distributions),
        )
        .route(
            "/profit-sharing/periods/{id}/distributions/export",
            get(handler::export_distributions),
        )
}
