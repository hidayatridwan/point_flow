use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{ApiResponse, CreatePayrollSlipRequest, ListQuery, PayrollSlipResponse},
};

use super::service;

pub async fn list_payroll_slips(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<PayrollSlipResponse>>>> {
    Ok(Json(ApiResponse::success(
        service::list_payroll_slips(&state.db, query).await?,
    )))
}

pub async fn get_payroll_slip(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<PayrollSlipResponse>>> {
    Ok(Json(ApiResponse::success(
        service::find_payroll_slip(&state.db, id).await?,
    )))
}

pub async fn create_payroll_slip(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreatePayrollSlipRequest>,
) -> AppResult<Json<ApiResponse<PayrollSlipResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::create_payroll_slip(&state.db, &user, payload).await?,
    )))
}
