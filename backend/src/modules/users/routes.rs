use axum::{Router, routing::get};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/users",
            get(handler::list_users).post(handler::create_user),
        )
        .route(
            "/users/{id}",
            get(handler::get_user)
                .put(handler::update_user)
                .delete(handler::delete_user),
        )
}
