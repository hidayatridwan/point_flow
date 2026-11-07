use axum::{
    Json,
    extract::{Multipart, Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::{AppError, AppResult},
    infra::s3::MAX_IMAGE_BYTES,
    models::{ApiResponse, CancelRequest, ListQuery, PaginationMeta, UploadResponse},
};

use super::{
    dto::{
        CheckoutRequest, CheckoutResponse, OnlineOrderDetailResponse, OnlineOrderResponse,
        PublicCatalogQuery, PublicCategory, PublicProductItem, StoreInfoResponse,
        UpdateStoreSettingsRequest,
    },
    service,
};

pub async fn list_public_products(
    State(state): State<AppState>,
    Query(query): Query<PublicCatalogQuery>,
) -> AppResult<Json<ApiResponse<Vec<PublicProductItem>, PaginationMeta>>> {
    let (items, meta) =
        service::list_catalog(&state.db, state.s3.as_deref(), query).await?;

    Ok(Json(ApiResponse::success_with_meta(items, meta)))
}

pub async fn list_public_categories(
    State(state): State<AppState>,
) -> AppResult<Json<ApiResponse<Vec<PublicCategory>>>> {
    Ok(Json(ApiResponse::success(
        service::list_categories(&state.db).await?,
    )))
}

pub async fn get_public_store_info(
    State(state): State<AppState>,
) -> AppResult<Json<ApiResponse<StoreInfoResponse>>> {
    Ok(Json(ApiResponse::success(
        service::get_store_info(&state.db, state.s3.as_deref()).await?,
    )))
}

pub async fn checkout(
    State(state): State<AppState>,
    Json(payload): Json<CheckoutRequest>,
) -> AppResult<Json<ApiResponse<CheckoutResponse>>> {
    Ok(Json(ApiResponse::success(
        service::checkout(&state.db, state.s3.as_deref(), payload).await?,
    )))
}

pub(crate) async fn read_single_image(multipart: &mut Multipart) -> AppResult<(Vec<u8>, String)> {
    let field = multipart
        .next_field()
        .await
        .map_err(|err| AppError::BadRequest(format!("invalid multipart body: {err}")))?
        .ok_or_else(|| AppError::BadRequest("no file uploaded".to_string()))?;
    let content_type = field
        .content_type()
        .map(str::to_string)
        .ok_or_else(|| AppError::Validation("missing image content type".to_string()))?;
    let bytes = field
        .bytes()
        .await
        .map_err(|err| AppError::BadRequest(format!("failed to read upload: {err}")))?
        .to_vec();
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(AppError::Validation(
            "image is too large (max 2 MB)".to_string(),
        ));
    }

    Ok((bytes, content_type))
}

pub async fn get_store_settings(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<StoreInfoResponse>>> {
    user.require_permission(&state.db, "online_orders").await?;
    Ok(Json(ApiResponse::success(
        service::get_store_settings(&state.db, state.s3.as_deref()).await?,
    )))
}

pub async fn update_store_settings(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<UpdateStoreSettingsRequest>,
) -> AppResult<Json<ApiResponse<StoreInfoResponse>>> {
    user.require_permission(&state.db, "online_orders").await?;
    Ok(Json(ApiResponse::success(
        service::update_store_settings(&state.db, state.s3.as_deref(), user.id, payload).await?,
    )))
}

pub async fn upload_qris_image(
    State(state): State<AppState>,
    user: AuthUser,
    mut multipart: Multipart,
) -> AppResult<Json<ApiResponse<UploadResponse>>> {
    user.require_permission(&state.db, "online_orders").await?;
    let (bytes, content_type) = read_single_image(&mut multipart).await?;
    Ok(Json(ApiResponse::success(
        service::upload_qris_image(
            &state.db,
            state.s3.as_deref(),
            user.id,
            bytes,
            &content_type,
        )
        .await?,
    )))
}

// ---------------------------------------------------------------------------
// Dashboard kasir (JWT + permission `online_orders`)
// ---------------------------------------------------------------------------

pub async fn list_orders(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<OnlineOrderResponse>, PaginationMeta>>> {
    user.require_permission(&state.db, "online_orders").await?;
    let (orders, meta) = service::list_orders(&state.db, query).await?;

    Ok(Json(ApiResponse::success_with_meta(orders, meta)))
}

pub async fn get_order(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<OnlineOrderDetailResponse>>> {
    user.require_permission(&state.db, "online_orders").await?;
    Ok(Json(ApiResponse::success(
        service::find_order(&state.db, id).await?,
    )))
}

pub async fn pack_order(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<OnlineOrderDetailResponse>>> {
    user.require_permission(&state.db, "online_orders").await?;
    Ok(Json(ApiResponse::success(
        service::pack_order(&state.db, user.id, id).await?,
    )))
}

pub async fn ready_order(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<OnlineOrderDetailResponse>>> {
    user.require_permission(&state.db, "online_orders").await?;
    Ok(Json(ApiResponse::success(
        service::ready_order(&state.db, user.id, id).await?,
    )))
}

pub async fn deliver_order(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<OnlineOrderDetailResponse>>> {
    user.require_permission(&state.db, "online_orders").await?;
    Ok(Json(ApiResponse::success(
        service::deliver_order(&state.db, user.id, id).await?,
    )))
}

pub async fn complete_order(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<OnlineOrderDetailResponse>>> {
    user.require_permission(&state.db, "online_orders").await?;
    Ok(Json(ApiResponse::success(
        service::complete_order(&state.db, user.id, id).await?,
    )))
}

pub async fn cancel_order(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CancelRequest>,
) -> AppResult<Json<ApiResponse<OnlineOrderDetailResponse>>> {
    user.require_permission(&state.db, "online_orders").await?;
    Ok(Json(ApiResponse::success(
        service::cancel_order(&state.db, user.id, id, payload.reason).await?,
    )))
}

pub async fn mark_order_paid(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<OnlineOrderDetailResponse>>> {
    user.require_permission(&state.db, "online_orders").await?;
    Ok(Json(ApiResponse::success(
        service::mark_order_paid(&state.db, user.id, id).await?,
    )))
}


