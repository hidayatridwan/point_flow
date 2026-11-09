use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, header},
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;

use crate::{
    app::AppState,
    auth::{AuthMember, AuthUser},
    error::{AppError, AppResult},
    models::{
        ApiResponse, ChangePasswordRequest, CreateMemberRequest, MemberLoginRequest, MemberQuery,
        MemberResponse, MessageResponse, ResetMemberPasswordRequest, UpdateMemberRequest,
    },
    services::member_auth,
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

// ---------------------------------------------------------------------------
// Auth member publik (storefront). Tanpa `AuthUser`; identitas dibuktikan
// dengan JWT member-scope via `AuthMember` untuk endpoint privat.
// ---------------------------------------------------------------------------

/// Nama cookie refresh khusus member — terpisah dari cookie staff agar dua
/// sesi (dashboard + storefront) tidak saling menimpa di browser yang sama.
pub const MEMBER_REFRESH_COOKIE_NAME: &str = "member_refresh_token";

pub async fn member_login(
    State(state): State<AppState>,
    Json(payload): Json<MemberLoginRequest>,
) -> AppResult<impl IntoResponse> {
    let session = member_auth::login(
        &state.db,
        payload,
        &state.jwt_secret,
        state.jwt_ttl_seconds,
        state.refresh_token_ttl_seconds,
    )
    .await?;

    Ok((
        member_refresh_token_headers(&state, &session.refresh_token)?,
        Json(ApiResponse::success(session.response)),
    ))
}

pub async fn member_refresh(
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<impl IntoResponse> {
    let refresh_token = jar
        .get(MEMBER_REFRESH_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string())
        .ok_or(AppError::Unauthorized)?;
    let session = member_auth::refresh(
        &state.db,
        &refresh_token,
        &state.jwt_secret,
        state.jwt_ttl_seconds,
        state.refresh_token_ttl_seconds,
    )
    .await?;

    Ok((
        member_refresh_token_headers(&state, &session.refresh_token)?,
        Json(ApiResponse::success(session.response)),
    ))
}

pub async fn member_logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<impl IntoResponse> {
    if let Some(cookie) = jar.get(MEMBER_REFRESH_COOKIE_NAME) {
        member_auth::logout(&state.db, cookie.value()).await?;
    }

    Ok((
        clear_member_refresh_token_headers(&state)?,
        Json(ApiResponse::success(MessageResponse {
            message: "logged out",
        })),
    ))
}

pub async fn member_me(
    State(state): State<AppState>,
    member: AuthMember,
) -> AppResult<Json<ApiResponse<MemberResponse>>> {
    Ok(Json(ApiResponse::success(
        member_auth::find_active_member(&state.db, member.id).await?,
    )))
}

pub async fn member_change_password(
    State(state): State<AppState>,
    member: AuthMember,
    Json(payload): Json<ChangePasswordRequest>,
) -> AppResult<impl IntoResponse> {
    member_auth::change_password(&state.db, member.id, payload).await?;

    Ok((
        clear_member_refresh_token_headers(&state)?,
        Json(ApiResponse::success(MessageResponse {
            message: "password changed",
        })),
    ))
}

fn member_refresh_token_headers(state: &AppState, refresh_token: &str) -> AppResult<HeaderMap> {
    set_member_cookie_header(build_member_refresh_token_cookie(
        state,
        refresh_token,
        false,
    ))
}

fn clear_member_refresh_token_headers(state: &AppState) -> AppResult<HeaderMap> {
    set_member_cookie_header(build_member_refresh_token_cookie(state, "", true))
}

fn set_member_cookie_header(value: String) -> AppResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    let value = HeaderValue::from_str(&value).map_err(|error| {
        AppError::Internal(format!("invalid member refresh token cookie: {error}"))
    })?;
    headers.insert(header::SET_COOKIE, value);
    Ok(headers)
}

fn build_member_refresh_token_cookie(state: &AppState, value: &str, clear: bool) -> String {
    let max_age = if clear {
        0
    } else {
        state.refresh_token_ttl_seconds
    };
    let mut parts = vec![
        format!("{MEMBER_REFRESH_COOKIE_NAME}={value}"),
        format!("Max-Age={max_age}"),
        "Path=/api/v1/members/auth".to_string(),
        "HttpOnly".to_string(),
        format!("SameSite={}", state.refresh_token_cookie_same_site),
    ];

    if let Some(domain) = &state.refresh_token_cookie_domain {
        parts.push(format!("Domain={domain}"));
    }
    if clear {
        parts.push("Expires=Thu, 01 Jan 1970 00:00:00 GMT".to_string());
    }
    if state.refresh_token_cookie_secure {
        parts.push("Secure".to_string());
    }

    parts.join("; ")
}
