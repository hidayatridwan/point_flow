use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        ApiResponse, CancelRequest, CreateFinancialCategoryRequest,
        CreateFinancialTransactionRequest, FinancialCategoryResponse, FinancialTransactionQuery,
        FinancialTransactionResponse, ListQuery, MessageResponse, UpdateFinancialCategoryRequest,
        UpdateFinancialTransactionRequest,
    },
};

use super::service;

pub async fn list_financial_categories(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ApiResponse<Vec<FinancialCategoryResponse>>>> {
    user.require_permission(&state.db, "finances").await?;
    Ok(Json(ApiResponse::success(
        service::list_financial_categories(&state.db, query).await?,
    )))
}

pub async fn get_financial_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<FinancialCategoryResponse>>> {
    user.require_permission(&state.db, "finances").await?;
    Ok(Json(ApiResponse::success(
        service::find_financial_category(&state.db, id).await?,
    )))
}

pub async fn create_financial_category(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateFinancialCategoryRequest>,
) -> AppResult<Json<ApiResponse<FinancialCategoryResponse>>> {
    user.require_permission(&state.db, "finances").await?;
    Ok(Json(ApiResponse::success(
        service::create_financial_category(&state.db, &user, payload).await?,
    )))
}

pub async fn update_financial_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateFinancialCategoryRequest>,
) -> AppResult<Json<ApiResponse<FinancialCategoryResponse>>> {
    user.require_permission(&state.db, "finances").await?;
    Ok(Json(ApiResponse::success(
        service::update_financial_category(&state.db, &user, id, payload).await?,
    )))
}

pub async fn delete_financial_category(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MessageResponse>>> {
    user.require_permission(&state.db, "finances").await?;
    service::delete_financial_category(&state.db, &user, id).await?;
    Ok(Json(ApiResponse::success(MessageResponse {
        message: "deleted",
    })))
}

pub async fn list_financial_transactions(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<FinancialTransactionQuery>,
) -> AppResult<Json<ApiResponse<Vec<FinancialTransactionResponse>>>> {
    user.require_permission(&state.db, "finances").await?;
    Ok(Json(ApiResponse::success(
        service::list_financial_transactions(&state.db, query).await?,
    )))
}

pub async fn get_financial_transaction(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<FinancialTransactionResponse>>> {
    user.require_permission(&state.db, "finances").await?;
    Ok(Json(ApiResponse::success(
        service::find_financial_transaction(&state.db, id).await?,
    )))
}

pub async fn create_financial_transaction(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateFinancialTransactionRequest>,
) -> AppResult<Json<ApiResponse<FinancialTransactionResponse>>> {
    user.require_permission(&state.db, "finances").await?;
    Ok(Json(ApiResponse::success(
        service::create_financial_transaction(&state.db, &user, payload).await?,
    )))
}

pub async fn update_financial_transaction(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateFinancialTransactionRequest>,
) -> AppResult<Json<ApiResponse<FinancialTransactionResponse>>> {
    user.require_permission(&state.db, "finances").await?;
    Ok(Json(ApiResponse::success(
        service::update_financial_transaction(&state.db, &user, id, payload).await?,
    )))
}

pub async fn cancel_financial_transaction(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CancelRequest>,
) -> AppResult<Json<ApiResponse<FinancialTransactionResponse>>> {
    user.require_permission(&state.db, "finances").await?;
    Ok(Json(ApiResponse::success(
        service::cancel_financial_transaction(&state.db, &user, id, payload).await?,
    )))
}
