use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        ApiResponse, CancelRequest, CashDrawerMovementQuery, CashDrawerMovementResponse,
        CashRegisterResponse, CashRegisterSessionResponse, CloseCashRegisterSessionRequest,
        CreateCashDrawerMovementRequest, CreateCashRegisterRequest, ListQuery, MessageResponse,
        OpenCashRegisterSessionRequest, UpdateCashRegisterRequest,
    },
};

use super::service;

pub async fn list_cash_registers(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<CashRegisterResponse>>>> {
    user.require_permission(&state.db, "master_data").await?;
    Ok(Json(ApiResponse::success(
        service::list_cash_registers(&state.db, query).await?,
    )))
}

pub async fn create_cash_register(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateCashRegisterRequest>,
) -> AppResult<Json<ApiResponse<CashRegisterResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    Ok(Json(ApiResponse::success(
        service::create_cash_register(&state.db, payload).await?,
    )))
}

pub async fn update_cash_register(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateCashRegisterRequest>,
) -> AppResult<Json<ApiResponse<CashRegisterResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    Ok(Json(ApiResponse::success(
        service::update_cash_register(&state.db, id, payload).await?,
    )))
}

pub async fn delete_cash_register(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    user.require_permission(&state.db, "master_data").await?;
    service::delete_cash_register(&state.db, id).await?;
    Ok(Json(ApiResponse::success(MessageResponse {
        message: "deleted",
    })))
}

pub async fn list_cash_register_sessions(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<CashRegisterSessionResponse>>>> {
    user.require_permission(&state.db, "point_flow").await?;
    Ok(Json(ApiResponse::success(
        service::list_cash_register_sessions(&state.db, query).await?,
    )))
}

pub async fn open_cash_register_session(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<OpenCashRegisterSessionRequest>,
) -> AppResult<Json<ApiResponse<CashRegisterSessionResponse>>> {
    user.require_permission(&state.db, "point_flow").await?;
    Ok(Json(ApiResponse::success(
        service::open_cash_register_session(&state.db, &user, payload).await?,
    )))
}

pub async fn close_cash_register_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CloseCashRegisterSessionRequest>,
) -> AppResult<Json<ApiResponse<CashRegisterSessionResponse>>> {
    user.require_permission(&state.db, "point_flow").await?;
    Ok(Json(ApiResponse::success(
        service::close_cash_register_session(&state.db, &user, id, payload).await?,
    )))
}

pub async fn list_cash_drawer_movements(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<CashDrawerMovementQuery>,
) -> AppResult<Json<ApiResponse<Vec<CashDrawerMovementResponse>>>> {
    user.require_permission(&state.db, "point_flow").await?;
    Ok(Json(ApiResponse::success(
        service::list_cash_drawer_movements(&state.db, query).await?,
    )))
}

pub async fn get_cash_drawer_movement(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<CashDrawerMovementResponse>>> {
    user.require_permission(&state.db, "point_flow").await?;
    Ok(Json(ApiResponse::success(
        service::find_cash_drawer_movement(&state.db, id).await?,
    )))
}

pub async fn create_cash_drawer_movement(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateCashDrawerMovementRequest>,
) -> AppResult<Json<ApiResponse<CashDrawerMovementResponse>>> {
    user.require_permission(&state.db, "point_flow").await?;
    Ok(Json(ApiResponse::success(
        service::create_cash_drawer_movement(&state.db, &user, payload).await?,
    )))
}

pub async fn cancel_cash_drawer_movement(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CancelRequest>,
) -> AppResult<Json<ApiResponse<CashDrawerMovementResponse>>> {
    user.require_permission(&state.db, "point_flow").await?;
    Ok(Json(ApiResponse::success(
        service::cancel_cash_drawer_movement(&state.db, &user, id, payload).await?,
    )))
}
