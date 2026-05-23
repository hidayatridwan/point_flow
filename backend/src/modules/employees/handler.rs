use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        ApiResponse, CreateEmployeeRequest, EmployeeResponse, ListQuery, MessageResponse,
        UpdateEmployeeRequest,
    },
};

use super::service;

pub async fn list_employees(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<EmployeeResponse>>>> {
    user.require_any_permission(&state.db, &["master_data", "payroll"])
        .await?;
    Ok(Json(ApiResponse::success(
        service::list_employees(&state.db, query).await?,
    )))
}

pub async fn get_employee(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<EmployeeResponse>>> {
    user.require_any_permission(&state.db, &["master_data", "payroll"])
        .await?;
    Ok(Json(ApiResponse::success(
        service::find_employee(&state.db, id).await?,
    )))
}

pub async fn create_employee(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateEmployeeRequest>,
) -> AppResult<Json<ApiResponse<EmployeeResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    Ok(Json(ApiResponse::success(
        service::create_employee(&state.db, payload).await?,
    )))
}

pub async fn update_employee(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateEmployeeRequest>,
) -> AppResult<Json<ApiResponse<EmployeeResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    Ok(Json(ApiResponse::success(
        service::update_employee(&state.db, id, payload).await?,
    )))
}

pub async fn delete_employee(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    service::delete_employee(&state.db, id).await?;
    Ok(Json(ApiResponse::success(MessageResponse {
        message: "deleted",
    })))
}
