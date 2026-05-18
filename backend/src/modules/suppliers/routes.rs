use axum::{Router, routing::get};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/suppliers",
            get(handler::list_suppliers).post(handler::create_supplier),
        )
        .route(
            "/suppliers/{id}",
            get(handler::get_supplier)
                .put(handler::update_supplier)
                .delete(handler::delete_supplier),
        )
}
