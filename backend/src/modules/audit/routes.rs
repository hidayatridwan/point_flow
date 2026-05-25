use axum::{Router, routing::get};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/audit-logs", get(handler::list_audit_logs))
        .route("/audit-logs/{id}", get(handler::get_audit_log))
}
