use axum::{Json, Router, routing::get};

use crate::{
    app::AppState,
    models::{ApiResponse, MessageResponse},
    modules::{
        audit, auth, cash_registers, employees, inventory, online_orders, payroll, procurement,
        products, reports, roles, sale_returns, sales, suppliers, users,
    },
};

pub fn public_router() -> Router<AppState> {
    online_orders::routes::public_routes()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .merge(auth::routes::routes())
        .merge(audit::routes::routes())
        .merge(cash_registers::routes::routes())
        .merge(users::routes::routes())
        .merge(products::routes::routes())
        .merge(suppliers::routes::routes())
        .merge(procurement::routes::routes())
        .merge(inventory::routes::routes())
        .merge(online_orders::routes::routes())
        .merge(sales::routes::routes())
        .merge(sale_returns::routes::routes())
        .merge(employees::routes::routes())
        .merge(payroll::routes::routes())
        .merge(reports::routes::routes())
        .merge(roles::routes::routes())
}

async fn health() -> Json<ApiResponse<MessageResponse>> {
    Json(ApiResponse::success(MessageResponse { message: "ok" }))
}
