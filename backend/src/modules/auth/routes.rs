use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/bootstrap-admin", post(handler::bootstrap_admin))
        .route("/auth/login", post(handler::login))
        .route("/auth/refresh", post(handler::refresh))
        .route("/auth/logout", post(handler::logout))
        .route("/auth/me", get(handler::me))
}
