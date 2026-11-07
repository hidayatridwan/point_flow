use axum::{
    Router,
    routing::{get, post},
};

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
        .route(
            "/products/{id}/photo",
            post(handler::upload_product_photo),
        )
        .route(
            "/units",
            get(handler::list_units).post(handler::create_unit),
        )
        .route(
            "/units/{id}",
            get(handler::get_unit)
                .put(handler::update_unit)
                .delete(handler::delete_unit),
        )
        .route(
            "/product-skus",
            get(handler::list_product_skus).post(handler::create_product_sku),
        )
        .route(
            "/product-skus/{id}",
            get(handler::get_product_sku)
                .put(handler::update_product_sku)
                .delete(handler::delete_product_sku),
        )
}
