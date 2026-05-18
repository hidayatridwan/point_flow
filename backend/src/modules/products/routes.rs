use axum::{Router, routing::get};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/product-categories",
            get(handler::list_categories).post(handler::create_category),
        )
        .route(
            "/product-categories/{id}",
            get(handler::get_category)
                .put(handler::update_category)
                .delete(handler::delete_category),
        )
        .route(
            "/products",
            get(handler::list_products).post(handler::create_product),
        )
        .route(
            "/products/{id}",
            get(handler::get_product)
                .put(handler::update_product)
                .delete(handler::delete_product),
        )
}
