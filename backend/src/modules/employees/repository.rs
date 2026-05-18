use sqlx::PgPool;

use crate::{
    error::AppResult,
    models::{CreateEmployeeRequest, EmployeeResponse, ListQuery, UpdateEmployeeRequest},
    services::master,
};

pub async fn list_employees(pool: &PgPool, query: ListQuery) -> AppResult<Vec<EmployeeResponse>> {
    master::list_employees(pool, query).await
}

pub async fn find_employee(pool: &PgPool, id: i64) -> AppResult<EmployeeResponse> {
    master::find_employee(pool, id).await
}

pub async fn create_employee(
    pool: &PgPool,
    request: CreateEmployeeRequest,
) -> AppResult<EmployeeResponse> {
    master::create_employee(pool, request).await
}

pub async fn update_employee(
    pool: &PgPool,
    id: i64,
    request: UpdateEmployeeRequest,
) -> AppResult<EmployeeResponse> {
    master::update_employee(pool, id, request).await
}

pub async fn delete_employee(pool: &PgPool, id: i64) -> AppResult<()> {
    master::delete_employee(pool, id).await
}
