use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, header},
    response::IntoResponse,
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::{AppError, AppResult},
    models::{
        ApiResponse, CancelRequest, CreateProfitSharingPeriodRequest, DistributionQuery, ListQuery,
        UpdateProfitSharingPeriodRequest,
    },
};

use super::service;

const PERMISSION: &str = "profit_sharing";

pub async fn list_periods(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<impl IntoResponse> {
    user.require_permission(&state.db, PERMISSION).await?;
    let result = service::list_periods(&state.db, query).await?;
    Ok(Json(ApiResponse::success_with_meta(
        result.data,
        result.meta,
    )))
}

pub async fn create_period(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateProfitSharingPeriodRequest>,
) -> AppResult<impl IntoResponse> {
    user.require_permission(&state.db, PERMISSION).await?;
    Ok(Json(ApiResponse::success(
        service::create_period(&state.db, &user, payload).await?,
    )))
}

pub async fn get_period(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    user.require_permission(&state.db, PERMISSION).await?;
    Ok(Json(ApiResponse::success(
        service::find_period_detail(&state.db, id).await?,
    )))
}

pub async fn update_period(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateProfitSharingPeriodRequest>,
) -> AppResult<impl IntoResponse> {
    user.require_permission(&state.db, PERMISSION).await?;
    Ok(Json(ApiResponse::success(
        service::update_period(&state.db, &user, id, payload).await?,
    )))
}

pub async fn calculate_period(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    user.require_permission(&state.db, PERMISSION).await?;
    Ok(Json(ApiResponse::success(
        service::calculate_period(&state.db, &user, id).await?,
    )))
}

pub async fn finalize_period(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    user.require_permission(&state.db, PERMISSION).await?;
    Ok(Json(ApiResponse::success(
        service::finalize_period(&state.db, &user, id).await?,
    )))
}

pub async fn cancel_period(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CancelRequest>,
) -> AppResult<impl IntoResponse> {
    user.require_permission(&state.db, PERMISSION).await?;
    Ok(Json(ApiResponse::success(
        service::cancel_period(&state.db, &user, id, payload).await?,
    )))
}

pub async fn list_distributions(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Query(query): Query<DistributionQuery>,
) -> AppResult<impl IntoResponse> {
    user.require_permission(&state.db, PERMISSION).await?;
    let result = service::list_distributions(&state.db, id, query).await?;
    Ok(Json(ApiResponse::success_with_meta(
        result.data,
        result.meta,
    )))
}

pub async fn export_distributions(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    user.require_permission(&state.db, PERMISSION).await?;
    let csv = service::export_distributions_csv(&state.db, id).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename=\"profit-sharing-{id}-distributions.csv\""
        ))
        .map_err(|error| AppError::Internal(format!("invalid export filename: {error}")))?,
    );

    Ok((headers, csv))
}
