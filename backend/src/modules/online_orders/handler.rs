use axum::{
    Json,
    extract::{Multipart, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::{AppError, AppResult},
    infra::s3::MAX_IMAGE_BYTES,
    models::{ApiResponse, PaginationMeta, UploadResponse},
};

use super::{
    dto::{CheckoutRequest, CheckoutResponse, PublicCatalogQuery, PublicCategory, PublicProductItem, StoreInfoResponse},
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


