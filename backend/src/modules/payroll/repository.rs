use sqlx::PgPool;

use crate::{
    auth::AuthUser,
    error::AppResult,
    models::{CreatePayrollSlipRequest, ListQuery, PayrollSlipResponse},
    services::documents,
};

pub async fn list_payroll_slips(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<PayrollSlipResponse>> {
    documents::list_payroll_slips(pool, query).await
}

pub async fn find_payroll_slip(pool: &PgPool, id: i64) -> AppResult<PayrollSlipResponse> {
    documents::find_payroll_slip(pool, id).await
}

pub async fn create_payroll_slip(
    pool: &PgPool,
    user: &AuthUser,
    request: CreatePayrollSlipRequest,
) -> AppResult<PayrollSlipResponse> {
    documents::create_payroll_slip(pool, user, request).await
}
