use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        CreateEmployeeRequest, EmployeeResponse, ListQuery, MessageResponse, UpdateEmployeeRequest,
    },
};

use super::service;

pub async fn list_employees(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<EmployeeResponse>>> {
    Ok(Json(service::list_employees(&state.db, query).await?))
}

pub async fn get_employee(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<EmployeeResponse>> {
    Ok(Json(service::find_employee(&state.db, id).await?))
}

pub async fn create_employee(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateEmployeeRequest>,
) -> AppResult<Json<EmployeeResponse>> {
    user.require_admin()?;
    Ok(Json(service::create_employee(&state.db, payload).await?))
}

pub async fn update_employee(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateEmployeeRequest>,
) -> AppResult<Json<EmployeeResponse>> {
    user.require_admin()?;
    Ok(Json(
        service::update_employee(&state.db, id, payload).await?,
    ))
}

pub async fn delete_employee(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<MessageResponse>> {
    user.require_admin()?;
    service::delete_employee(&state.db, id).await?;
    Ok(Json(MessageResponse { message: "deleted" }))
}
