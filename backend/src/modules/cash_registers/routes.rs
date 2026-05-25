use axum::{
    Router,
    routing::{get, post, put},
};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/cash-registers",
            get(handler::list_cash_registers).post(handler::create_cash_register),
        )
        .route(
            "/cash-registers/{id}",
            put(handler::update_cash_register).delete(handler::delete_cash_register),
        )
        .route(
            "/cash-register-sessions",
            get(handler::list_cash_register_sessions),
        )
        .route(
            "/cash-register-sessions/open",
            post(handler::open_cash_register_session),
        )
        .route(
            "/cash-register-sessions/{id}/close",
            post(handler::close_cash_register_session),
        )
        .route(
            "/cash-drawer-movements",
            get(handler::list_cash_drawer_movements).post(handler::create_cash_drawer_movement),
        )
        .route(
            "/cash-drawer-movements/{id}",
            get(handler::get_cash_drawer_movement),
        )
        .route(
            "/cash-drawer-movements/{id}/cancel",
            post(handler::cancel_cash_drawer_movement),
        )
}
