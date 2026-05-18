use axum::{Router, routing::get};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/employees",
            get(handler::list_employees).post(handler::create_employee),
        )
        .route(
            "/employees/{id}",
            get(handler::get_employee)
                .put(handler::update_employee)
                .delete(handler::delete_employee),
        )
}
