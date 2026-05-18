use sqlx::PgPool;

use crate::{
    error::AppResult,
    models::{CreateSupplierRequest, ListQuery, SupplierResponse, UpdateSupplierRequest},
    services::master,
};

pub async fn list_suppliers(pool: &PgPool, query: ListQuery) -> AppResult<Vec<SupplierResponse>> {
    master::list_suppliers(pool, query).await
}

pub async fn find_supplier(pool: &PgPool, id: i64) -> AppResult<SupplierResponse> {
    master::find_supplier(pool, id).await
}

pub async fn create_supplier(
    pool: &PgPool,
    request: CreateSupplierRequest,
) -> AppResult<SupplierResponse> {
    master::create_supplier(pool, request).await
}

pub async fn update_supplier(
    pool: &PgPool,
    id: i64,
    request: UpdateSupplierRequest,
) -> AppResult<SupplierResponse> {
    master::update_supplier(pool, id, request).await
}

pub async fn delete_supplier(pool: &PgPool, id: i64) -> AppResult<()> {
    master::delete_supplier(pool, id).await
}
