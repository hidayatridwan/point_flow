use sqlx::PgPool;

use crate::{
    auth::AuthUser,
    error::AppResult,
    models::{
        CancelRequest, CashDrawerMovementQuery, CashDrawerMovementResponse, CashRegisterResponse,
        CashRegisterSessionResponse, CloseCashRegisterSessionRequest,
        CreateCashDrawerMovementRequest, CreateCashRegisterRequest, ListQuery,
        OpenCashRegisterSessionRequest, UpdateCashRegisterRequest,
    },
    services::cash_registers,
};

pub async fn list_cash_registers(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<CashRegisterResponse>> {
    cash_registers::list_cash_registers(pool, query).await
}

pub async fn create_cash_register(
    pool: &PgPool,
    request: CreateCashRegisterRequest,
) -> AppResult<CashRegisterResponse> {
    cash_registers::create_cash_register(pool, request).await
}

pub async fn update_cash_register(
    pool: &PgPool,
    id: i64,
    request: UpdateCashRegisterRequest,
) -> AppResult<CashRegisterResponse> {
    cash_registers::update_cash_register(pool, id, request).await
}

pub async fn delete_cash_register(pool: &PgPool, id: i64) -> AppResult<()> {
    cash_registers::delete_cash_register(pool, id).await
}

pub async fn list_cash_register_sessions(
    pool: &PgPool,
    query: ListQuery,
) -> AppResult<Vec<CashRegisterSessionResponse>> {
    cash_registers::list_cash_register_sessions(pool, query).await
}

pub async fn open_cash_register_session(
    pool: &PgPool,
    user: &AuthUser,
    request: OpenCashRegisterSessionRequest,
) -> AppResult<CashRegisterSessionResponse> {
    cash_registers::open_cash_register_session(pool, user, request).await
}

pub async fn close_cash_register_session(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CloseCashRegisterSessionRequest,
) -> AppResult<CashRegisterSessionResponse> {
    cash_registers::close_cash_register_session(pool, user, id, request).await
}

pub async fn list_cash_drawer_movements(
    pool: &PgPool,
    query: CashDrawerMovementQuery,
) -> AppResult<Vec<CashDrawerMovementResponse>> {
    cash_registers::list_cash_drawer_movements(pool, query).await
}

pub async fn find_cash_drawer_movement(
    pool: &PgPool,
    id: i64,
) -> AppResult<CashDrawerMovementResponse> {
    cash_registers::find_cash_drawer_movement(pool, id).await
}

pub async fn create_cash_drawer_movement(
    pool: &PgPool,
    user: &AuthUser,
    request: CreateCashDrawerMovementRequest,
) -> AppResult<CashDrawerMovementResponse> {
    cash_registers::create_cash_drawer_movement(pool, user, request).await
}

pub async fn cancel_cash_drawer_movement(
    pool: &PgPool,
    user: &AuthUser,
    id: i64,
    request: CancelRequest,
) -> AppResult<CashDrawerMovementResponse> {
    cash_registers::cancel_cash_drawer_movement(pool, user, id, request).await
}
