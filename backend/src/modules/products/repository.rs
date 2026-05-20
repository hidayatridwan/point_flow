use sqlx::PgPool;

use crate::{
    error::AppResult,
    models::{
        CreateProductCategoryRequest, CreateProductRequest, CreateProductSkuRequest,
        CreateUnitRequest, ListQuery, Paginated, ProductCategoryResponse, ProductResponse,
        ProductSkuResponse, UnitResponse, UpdateProductCategoryRequest, UpdateProductRequest,
        UpdateProductSkuRequest, UpdateUnitRequest,
    },
    services::master,
};

pub async fn list_categories(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Paginated<ProductCategoryResponse>> {
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

pub async fn list_units(pool: &PgPool, query: ListQuery) -> AppResult<Vec<UnitResponse>> {
    master::list_units(pool, query).await
}

pub async fn find_unit(pool: &PgPool, id: i64) -> AppResult<UnitResponse> {
    master::find_unit(pool, id).await
}

pub async fn create_unit(pool: &PgPool, request: CreateUnitRequest) -> AppResult<UnitResponse> {
    master::create_unit(pool, request).await
}

pub async fn update_unit(
    pool: &PgPool,
    id: i64,
    request: UpdateUnitRequest,
) -> AppResult<UnitResponse> {
    master::update_unit(pool, id, request).await
}

pub async fn delete_unit(pool: &PgPool, id: i64) -> AppResult<()> {
    master::delete_unit(pool, id).await
}

pub async fn list_product_skus(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<ProductSkuResponse>> {
    master::list_product_skus(pool, query).await
}

pub async fn find_product_sku(pool: &PgPool, id: i64) -> AppResult<ProductSkuResponse> {
    master::find_product_sku(pool, id).await
}

pub async fn create_product_sku(
    pool: &PgPool,
    request: CreateProductSkuRequest,
) -> AppResult<ProductSkuResponse> {
    master::create_product_sku(pool, request).await
}

pub async fn update_product_sku(
    pool: &PgPool,
    id: i64,
    request: UpdateProductSkuRequest,
) -> AppResult<ProductSkuResponse> {
    master::update_product_sku(pool, id, request).await
}

pub async fn delete_product_sku(pool: &PgPool, id: i64) -> AppResult<()> {
    master::delete_product_sku(pool, id).await
}
