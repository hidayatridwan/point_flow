use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/financial-categories",
            get(handler::list_financial_categories).post(handler::create_financial_category),
        )
        .route(
            "/financial-categories/{id}",
            get(handler::get_financial_category)
                .put(handler::update_financial_category)
                .delete(handler::delete_financial_category),
        )
        .route(
            "/financial-transactions",
            get(handler::list_financial_transactions).post(handler::create_financial_transaction),
        )
        .route(
            "/financial-transactions/{id}",
            get(handler::get_financial_transaction).put(handler::update_financial_transaction),
        )
        .route(
            "/financial-transactions/{id}/cancel",
            post(handler::cancel_financial_transaction),
        )
}
