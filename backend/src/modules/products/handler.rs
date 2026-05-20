use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        ApiResponse, CreateProductCategoryRequest, CreateProductRequest, CreateProductSkuRequest,
        CreateUnitRequest, ListQuery, MessageResponse, PaginationMeta, ProductCategoryResponse,
        ProductResponse, ProductSkuResponse, UnitResponse, UpdateProductCategoryRequest,
        UpdateProductRequest, UpdateProductSkuRequest, UpdateUnitRequest,
    },
};

use super::service;

pub async fn list_categories(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<ProductCategoryResponse>, PaginationMeta>>> {
    let categories = service::list_categories(&state.db, query).await?;

    Ok(Json(ApiResponse::success_with_meta(
        categories.data,
        categories.meta,
    )))
}

pub async fn get_category(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<ProductCategoryResponse>>> {
    Ok(Json(ApiResponse::success(
        service::find_category(&state.db, id).await?,
    )))
}

pub async fn create_category(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateProductCategoryRequest>,
) -> AppResult<Json<ApiResponse<ProductCategoryResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::create_category(&state.db, payload).await?,
    )))
}

pub async fn update_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateProductCategoryRequest>,
) -> AppResult<Json<ApiResponse<ProductCategoryResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::update_category(&state.db, id, payload).await?,
    )))
}

pub async fn delete_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    user.require_admin()?;
    service::delete_category(&state.db, id).await?;
    Ok(Json(ApiResponse::success(MessageResponse {
        message: "deleted",
    })))
}

pub async fn list_products(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<ProductResponse>>>> {
    Ok(Json(ApiResponse::success(
        service::list_products(&state.db, query).await?,
    )))
}

pub async fn get_product(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<ProductResponse>>> {
    Ok(Json(ApiResponse::success(
        service::find_product(&state.db, id).await?,
    )))
}

pub async fn create_product(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateProductRequest>,
) -> AppResult<Json<ApiResponse<ProductResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::create_product(&state.db, payload).await?,
    )))
}

pub async fn update_product(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateProductRequest>,
) -> AppResult<Json<ApiResponse<ProductResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::update_product(&state.db, id, payload).await?,
    )))
}

pub async fn delete_product(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    user.require_admin()?;
    service::delete_product(&state.db, id).await?;
    Ok(Json(ApiResponse::success(MessageResponse {
        message: "deleted",
    })))
}

pub async fn list_units(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<UnitResponse>>>> {
    Ok(Json(ApiResponse::success(
        service::list_units(&state.db, query).await?,
    )))
}

pub async fn get_unit(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<UnitResponse>>> {
    Ok(Json(ApiResponse::success(
        service::find_unit(&state.db, id).await?,
    )))
}

pub async fn create_unit(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateUnitRequest>,
) -> AppResult<Json<ApiResponse<UnitResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::create_unit(&state.db, payload).await?,
    )))
}

pub async fn update_unit(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateUnitRequest>,
) -> AppResult<Json<ApiResponse<UnitResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::update_unit(&state.db, id, payload).await?,
    )))
}

pub async fn delete_unit(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    user.require_admin()?;
    service::delete_unit(&state.db, id).await?;
    Ok(Json(ApiResponse::success(MessageResponse {
        message: "deleted",
    })))
}

pub async fn list_product_skus(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<ProductSkuResponse>>>> {
    Ok(Json(ApiResponse::success(
        service::list_product_skus(&state.db, query).await?,
    )))
}

pub async fn get_product_sku(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<ProductSkuResponse>>> {
    Ok(Json(ApiResponse::success(
        service::find_product_sku(&state.db, id).await?,
    )))
}

pub async fn create_product_sku(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateProductSkuRequest>,
) -> AppResult<Json<ApiResponse<ProductSkuResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::create_product_sku(&state.db, payload).await?,
    )))
}

pub async fn update_product_sku(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateProductSkuRequest>,
) -> AppResult<Json<ApiResponse<ProductSkuResponse>>> {
    user.require_admin()?;
    Ok(Json(ApiResponse::success(
        service::update_product_sku(&state.db, id, payload).await?,
    )))
}

pub async fn delete_product_sku(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    user.require_admin()?;
    service::delete_product_sku(&state.db, id).await?;
    Ok(Json(ApiResponse::success(MessageResponse {
        message: "deleted",
    })))
}
