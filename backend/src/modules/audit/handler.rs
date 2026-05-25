use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{ApiResponse, AuditLogQuery, AuditLogResponse},
};

use super::service;

pub async fn list_audit_logs(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<AuditLogQuery>,
) -> AppResult<Json<ApiResponse<Vec<AuditLogResponse>>>> {
    user.require_permission(&state.db, "users").await?;
    Ok(Json(ApiResponse::success(
        service::list_audit_logs(&state.db, query).await?,
    )))
}

pub async fn get_audit_log(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<AuditLogResponse>>> {
    user.require_permission(&state.db, "users").await?;
    Ok(Json(ApiResponse::success(
        service::find_audit_log(&state.db, id).await?,
    )))
}
