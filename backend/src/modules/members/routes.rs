use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/members",
            get(handler::list_members).post(handler::create_member),
        )
        .route(
            "/members/{id}",
            get(handler::get_member)
                .put(handler::update_member)
                .delete(handler::delete_member),
        )
        .route(
            "/members/{id}/reset-password",
            post(handler::reset_member_password),
        )
}
