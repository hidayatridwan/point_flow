use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        CreateProductCategoryRequest, CreateProductRequest, CreateProductSkuRequest,
        CreateUnitRequest, ListQuery, MessageResponse, ProductCategoryResponse, ProductResponse,
        ProductSkuResponse, UnitResponse, UpdateProductCategoryRequest, UpdateProductRequest,
        UpdateProductSkuRequest, UpdateUnitRequest,
    },
};

use super::service;

pub async fn list_categories(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<ProductCategoryResponse>>> {
    Ok(Json(service::list_categories(&state.db, query).await?))
}

pub async fn get_category(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ProductCategoryResponse>> {
    Ok(Json(service::find_category(&state.db, id).await?))
}

pub async fn create_category(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateProductCategoryRequest>,
) -> AppResult<Json<ProductCategoryResponse>> {
    user.require_admin()?;
    Ok(Json(service::create_category(&state.db, payload).await?))
}

pub async fn update_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateProductCategoryRequest>,
) -> AppResult<Json<ProductCategoryResponse>> {
    user.require_admin()?;
    Ok(Json(
        service::update_category(&state.db, id, payload).await?,
    ))
}

pub async fn delete_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<MessageResponse>> {
    user.require_admin()?;
    service::delete_category(&state.db, id).await?;
    Ok(Json(MessageResponse { message: "deleted" }))
}

pub async fn list_products(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<ProductResponse>>> {
    Ok(Json(service::list_products(&state.db, query).await?))
}

pub async fn get_product(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ProductResponse>> {
    Ok(Json(service::find_product(&state.db, id).await?))
}

pub async fn create_product(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateProductRequest>,
) -> AppResult<Json<ProductResponse>> {
    user.require_admin()?;
    Ok(Json(service::create_product(&state.db, payload).await?))
}

pub async fn update_product(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateProductRequest>,
) -> AppResult<Json<ProductResponse>> {
    user.require_admin()?;
    Ok(Json(service::update_product(&state.db, id, payload).await?))
}

pub async fn delete_product(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<MessageResponse>> {
    user.require_admin()?;
    service::delete_product(&state.db, id).await?;
    Ok(Json(MessageResponse { message: "deleted" }))
}

pub async fn list_units(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<UnitResponse>>> {
    Ok(Json(service::list_units(&state.db, query).await?))
}

pub async fn get_unit(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<UnitResponse>> {
    Ok(Json(service::find_unit(&state.db, id).await?))
}

pub async fn create_unit(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateUnitRequest>,
) -> AppResult<Json<UnitResponse>> {
    user.require_admin()?;
    Ok(Json(service::create_unit(&state.db, payload).await?))
}

pub async fn update_unit(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateUnitRequest>,
) -> AppResult<Json<UnitResponse>> {
    user.require_admin()?;
    Ok(Json(service::update_unit(&state.db, id, payload).await?))
}

pub async fn delete_unit(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<MessageResponse>> {
    user.require_admin()?;
    service::delete_unit(&state.db, id).await?;
    Ok(Json(MessageResponse { message: "deleted" }))
}

pub async fn list_product_skus(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<ProductSkuResponse>>> {
    Ok(Json(service::list_product_skus(&state.db, query).await?))
}

pub async fn get_product_sku(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ProductSkuResponse>> {
    Ok(Json(service::find_product_sku(&state.db, id).await?))
}

pub async fn create_product_sku(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateProductSkuRequest>,
) -> AppResult<Json<ProductSkuResponse>> {
    user.require_admin()?;
    Ok(Json(service::create_product_sku(&state.db, payload).await?))
}

pub async fn update_product_sku(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateProductSkuRequest>,
) -> AppResult<Json<ProductSkuResponse>> {
    user.require_admin()?;
    Ok(Json(
        service::update_product_sku(&state.db, id, payload).await?,
    ))
}

pub async fn delete_product_sku(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<MessageResponse>> {
    user.require_admin()?;
    service::delete_product_sku(&state.db, id).await?;
    Ok(Json(MessageResponse { message: "deleted" }))
}
