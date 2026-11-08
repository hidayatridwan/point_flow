use axum::{
    Json,
    extract::{Multipart, Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    infra::s3::ObjectStorage,
    models::{
        ApiResponse, CreateProductCategoryRequest, CreateProductRequest, CreateProductSkuRequest,
        CreateUnitRequest, ListQuery, MessageResponse, PaginationMeta, ProductCategoryResponse,
        ProductResponse, ProductSkuResponse, UnitResponse, UpdateProductCategoryRequest,
        UpdateProductRequest, UpdateProductSkuRequest, UpdateUnitRequest, UploadResponse,
    },
};

/// Isi `photo_url` dari object key agar dashboard bisa menampilkan foto produk.
fn with_photo_url(mut product: ProductResponse, s3: Option<&ObjectStorage>) -> ProductResponse {
    product.photo_url = product
        .image_url
        .as_deref()
        .filter(|key| !key.is_empty())
        .and_then(|key| s3.map(|storage| storage.public_url(key)));
    product
}

use super::service;

pub async fn list_categories(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<ProductCategoryResponse>, PaginationMeta>>> {
    user.require_permission(&state.db, "master_data").await?;
    let categories = service::list_categories(&state.db, query).await?;

    Ok(Json(ApiResponse::success_with_meta(
        categories.data,
        categories.meta,
    )))
}

pub async fn get_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<ProductCategoryResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    Ok(Json(ApiResponse::success(
        service::find_category(&state.db, id).await?,
    )))
}

pub async fn create_category(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateProductCategoryRequest>,
) -> AppResult<Json<ApiResponse<ProductCategoryResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    let category = service::create_category(&state.db, payload).await?;
    state.public_cache.invalidate();
    Ok(Json(ApiResponse::success(category)))
}

pub async fn update_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateProductCategoryRequest>,
) -> AppResult<Json<ApiResponse<ProductCategoryResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    let category = service::update_category(&state.db, id, payload).await?;
    state.public_cache.invalidate();
    Ok(Json(ApiResponse::success(category)))
}

pub async fn delete_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    service::delete_category(&state.db, id).await?;
    state.public_cache.invalidate();
    Ok(Json(ApiResponse::success(MessageResponse {
        message: "deleted",
    })))
}

pub async fn list_products(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<ProductResponse>>>> {
    user.require_permission(&state.db, "master_data").await?;
    let products = service::list_products(&state.db, query).await?;
    let products = products
        .into_iter()
        .map(|product| with_photo_url(product, state.s3.as_deref()))
        .collect();
    Ok(Json(ApiResponse::success(products)))
}

pub async fn get_product(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<ProductResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    let product = service::find_product(&state.db, id).await?;
    Ok(Json(ApiResponse::success(with_photo_url(
        product,
        state.s3.as_deref(),
    ))))
}

pub async fn create_product(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateProductRequest>,
) -> AppResult<Json<ApiResponse<ProductResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    let product = service::create_product(&state.db, payload).await?;
    state.public_cache.invalidate();
    Ok(Json(ApiResponse::success(with_photo_url(
        product,
        state.s3.as_deref(),
    ))))
}

pub async fn update_product(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateProductRequest>,
) -> AppResult<Json<ApiResponse<ProductResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    let product = service::update_product(&state.db, id, payload).await?;
    state.public_cache.invalidate();
    Ok(Json(ApiResponse::success(with_photo_url(
        product,
        state.s3.as_deref(),
    ))))
}

pub async fn delete_product(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    service::delete_product(&state.db, id).await?;
    state.public_cache.invalidate();
    Ok(Json(ApiResponse::success(MessageResponse {
        message: "deleted",
    })))
}

pub async fn upload_product_photo(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    mut multipart: Multipart,
) -> AppResult<Json<ApiResponse<UploadResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    let (bytes, content_type) =
        super::super::online_orders::handler::read_single_image(&mut multipart).await?;
    let uploaded =
        service::upload_product_photo(&state.db, state.s3.as_deref(), id, bytes, &content_type)
            .await?;
    // Foto baru tampil di katalog publik; bersihkan cache-nya.
    state.public_cache.invalidate();
    Ok(Json(ApiResponse::success(uploaded)))
}

pub async fn list_units(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<UnitResponse>>>> {
    user.require_permission(&state.db, "master_data").await?;
    Ok(Json(ApiResponse::success(
        service::list_units(&state.db, query).await?,
    )))
}

pub async fn get_unit(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<UnitResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    Ok(Json(ApiResponse::success(
        service::find_unit(&state.db, id).await?,
    )))
}

pub async fn create_unit(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateUnitRequest>,
) -> AppResult<Json<ApiResponse<UnitResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    let unit = service::create_unit(&state.db, payload).await?;
    // Kode unit ikut tampil di katalog publik; bersihkan cache-nya.
    state.public_cache.invalidate();
    Ok(Json(ApiResponse::success(unit)))
}

pub async fn update_unit(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateUnitRequest>,
) -> AppResult<Json<ApiResponse<UnitResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    let unit = service::update_unit(&state.db, id, payload).await?;
    state.public_cache.invalidate();
    Ok(Json(ApiResponse::success(unit)))
}

pub async fn delete_unit(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    service::delete_unit(&state.db, id).await?;
    state.public_cache.invalidate();
    Ok(Json(ApiResponse::success(MessageResponse {
        message: "deleted",
    })))
}

pub async fn list_product_skus(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<ProductSkuResponse>>>> {
    user.require_any_permission(
        &state.db,
        &["master_data", "point_flow", "procurement", "inventory"],
    )
    .await?;
    Ok(Json(ApiResponse::success(
        service::list_product_skus(&state.db, query).await?,
    )))
}

pub async fn get_product_sku(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<ProductSkuResponse>>> {
    user.require_any_permission(
        &state.db,
        &["master_data", "point_flow", "procurement", "inventory"],
    )
    .await?;
    Ok(Json(ApiResponse::success(
        service::find_product_sku(&state.db, id).await?,
    )))
}

pub async fn create_product_sku(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateProductSkuRequest>,
) -> AppResult<Json<ApiResponse<ProductSkuResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    let sku = service::create_product_sku(&state.db, payload).await?;
    state.public_cache.invalidate();
    Ok(Json(ApiResponse::success(sku)))
}

pub async fn update_product_sku(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateProductSkuRequest>,
) -> AppResult<Json<ApiResponse<ProductSkuResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    let sku = service::update_product_sku(&state.db, id, payload).await?;
    state.public_cache.invalidate();
    Ok(Json(ApiResponse::success(sku)))
}

pub async fn delete_product_sku(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    service::delete_product_sku(&state.db, id).await?;
    state.public_cache.invalidate();
    Ok(Json(ApiResponse::success(MessageResponse {
        message: "deleted",
    })))
}
