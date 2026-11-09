use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    app::AppState,
    auth::AuthUser,
    error::AppResult,
    models::{
        ApiResponse, CreateMemberRequest, MemberQuery, MemberResponse, ResetMemberPasswordRequest,
        UpdateMemberRequest,
    },
};

use super::service;

pub async fn list_members(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<MemberQuery>,
) -> AppResult<Json<ApiResponse<Vec<MemberResponse>>>> {
    user.require_permission(&state.db, "members").await?;
    Ok(Json(ApiResponse::success(
        service::list_members(&state.db, query).await?,
    )))
}

pub async fn get_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MemberResponse>>> {
    user.require_permission(&state.db, "members").await?;
    Ok(Json(ApiResponse::success(
        service::find_member(&state.db, id).await?,
    )))
}

pub async fn create_member(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateMemberRequest>,
) -> AppResult<Json<ApiResponse<MemberResponse>>> {
    user.require_permission(&state.db, "members").await?;
    Ok(Json(ApiResponse::success(
        service::create_member(&state.db, &user, payload).await?,
    )))
}

pub async fn update_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateMemberRequest>,
) -> AppResult<Json<ApiResponse<MemberResponse>>> {
    user.require_permission(&state.db, "members").await?;
    Ok(Json(ApiResponse::success(
        service::update_member(&state.db, &user, id, payload).await?,
    )))
}

pub async fn delete_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MemberResponse>>> {
    user.require_permission(&state.db, "members").await?;
    Ok(Json(ApiResponse::success(
        service::delete_member(&state.db, &user, id).await?,
    )))
}

pub async fn reset_member_password(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<ResetMemberPasswordRequest>,
) -> AppResult<Json<ApiResponse<MemberResponse>>> {
    user.require_permission(&state.db, "members").await?;
    Ok(Json(ApiResponse::success(
        service::reset_member_password(&state.db, &user, id, payload).await?,
    )))
}
