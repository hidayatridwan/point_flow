use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{CreatePayrollSlipRequest, ListQuery, PayrollSlipResponse},
};

use super::service;

pub async fn list_payroll_slips(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<PayrollSlipResponse>>> {
    Ok(Json(service::list_payroll_slips(&state.db, query).await?))
}

pub async fn get_payroll_slip(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<PayrollSlipResponse>> {
    Ok(Json(service::find_payroll_slip(&state.db, id).await?))
}

pub async fn create_payroll_slip(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreatePayrollSlipRequest>,
) -> AppResult<Json<PayrollSlipResponse>> {
    user.require_admin()?;
    Ok(Json(
        service::create_payroll_slip(&state.db, &user, payload).await?,
    ))
}
