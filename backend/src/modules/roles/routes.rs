use axum::{
    Router,
    routing::{get, put},
};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/roles",
            get(handler::list_roles).post(handler::create_role),
        )
        .route(
            "/roles/{id}",
            get(handler::get_role)
                .put(handler::update_role)
                .delete(handler::delete_role),
        )
        .route(
            "/roles/{id}/permissions",
            put(handler::replace_role_permissions),
        )
        .route("/permissions", get(handler::list_permissions))
}
