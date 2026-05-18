use sqlx::PgPool;

use crate::{
    error::AppResult,
    models::{
        CreateProductCategoryRequest, CreateProductRequest, ListQuery, ProductCategoryResponse,
        ProductResponse, UpdateProductCategoryRequest, UpdateProductRequest,
    },
    services::master,
};

pub async fn list_categories(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<ProductCategoryResponse>> {
    master::list_categories(pool, query).await
}

pub async fn find_category(pool: &PgPool, id: i64) -> AppResult<ProductCategoryResponse> {
    master::find_category(pool, id).await
}

pub async fn create_category(
    pool: &PgPool,
    request: CreateProductCategoryRequest,
) -> AppResult<ProductCategoryResponse> {
    master::create_category(pool, request).await
}

pub async fn update_category(
    pool: &PgPool,
    id: i64,
    request: UpdateProductCategoryRequest,
) -> AppResult<ProductCategoryResponse> {
    master::update_category(pool, id, request).await
}

pub async fn delete_category(pool: &PgPool, id: i64) -> AppResult<()> {
    master::delete_category(pool, id).await
}

pub async fn list_products(pool: &PgPool, query: ListQuery) -> AppResult<Vec<ProductResponse>> {
    master::list_products(pool, query).await
}

pub async fn find_product(pool: &PgPool, id: i64) -> AppResult<ProductResponse> {
    master::find_product(pool, id).await
}

pub async fn create_product(
    pool: &PgPool,
    request: CreateProductRequest,
) -> AppResult<ProductResponse> {
    master::create_product(pool, request).await
}

pub async fn update_product(
    pool: &PgPool,
    id: i64,
    request: UpdateProductRequest,
) -> AppResult<ProductResponse> {
    master::update_product(pool, id, request).await
}

pub async fn delete_product(pool: &PgPool, id: i64) -> AppResult<()> {
    master::delete_product(pool, id).await
}
