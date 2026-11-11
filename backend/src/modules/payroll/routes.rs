use axum::{Router, routing::{get, post}};

use crate::app::AppState;

use super::handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/payroll-slips",
            get(handler::list_payroll_slips).post(handler::create_payroll_slip),
        )
        .route("/payroll-slips/{id}", get(handler::get_payroll_slip))
        .route("/payroll-slips/{id}/cancel", post(handler::cancel_payroll_slip))
}
