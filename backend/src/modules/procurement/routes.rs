use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/purchase-orders",
            get(handler::list_purchase_orders).post(handler::create_purchase_order),
        )
        .route("/purchase-orders/{id}", get(handler::get_purchase_order))
        .route(
            "/purchase-orders/{id}/order",
            post(handler::order_purchase_order),
        )
        .route(
            "/purchase-orders/{id}/cancel",
            post(handler::cancel_purchase_order),
        )
        .route(
            "/goods-receipts",
            get(handler::list_goods_receipts).post(handler::create_goods_receipt),
        )
        .route("/goods-receipts/{id}", get(handler::get_goods_receipt))
        .route(
            "/goods-receipts/{id}/post",
            post(handler::post_goods_receipt),
        )
        .route(
            "/goods-receipts/{id}/cancel",
            post(handler::cancel_goods_receipt),
        )
}
