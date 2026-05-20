use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, header},
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;

use crate::{
    app::AppState,
    auth::AuthUser,
    error::{AppError, AppResult},
    models::{ApiResponse, CreateUserRequest, LoginRequest, MessageResponse, UserResponse},
};

use super::service;

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    let session = service::login(
        &state.db,
        payload,
        &state.jwt_secret,
        state.jwt_ttl_seconds,
        state.refresh_token_ttl_seconds,
    )
    .await?;

    Ok((
        refresh_token_headers(&state, &session.refresh_token)?,
        Json(ApiResponse::success(session.response)),
    ))
}

pub async fn bootstrap_admin(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<impl IntoResponse> {
    let session = service::bootstrap_admin(
        &state.db,
        payload,
        &state.jwt_secret,
        state.jwt_ttl_seconds,
        state.refresh_token_ttl_seconds,
    )
    .await?;

    Ok((
        refresh_token_headers(&state, &session.refresh_token)?,
        Json(ApiResponse::success(session.response)),
    ))
}

pub async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<impl IntoResponse> {
    let refresh_token = jar
        .get(&state.refresh_token_cookie_name)
        .map(|cookie| cookie.value().to_string())
        .ok_or(AppError::Unauthorized)?;
    let session = service::refresh(
        &state.db,
        &refresh_token,
        &state.jwt_secret,
        state.jwt_ttl_seconds,
        state.refresh_token_ttl_seconds,
    )
    .await?;

    Ok((
        refresh_token_headers(&state, &session.refresh_token)?,
        Json(ApiResponse::success(session.response)),
    ))
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> AppResult<impl IntoResponse> {
    if let Some(cookie) = jar.get(&state.refresh_token_cookie_name) {
        service::logout(&state.db, cookie.value()).await?;
    }

    Ok((
        clear_refresh_token_headers(&state)?,
        Json(ApiResponse::success(MessageResponse {
            message: "logged out",
        })),
    ))
}

pub async fn me(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    Ok(Json(ApiResponse::success(
        service::find_user(&state.db, user.id).await?,
    )))
}

fn refresh_token_headers(state: &AppState, refresh_token: &str) -> AppResult<HeaderMap> {
    set_cookie_header(build_refresh_token_cookie(state, refresh_token, false))
}

fn clear_refresh_token_headers(state: &AppState) -> AppResult<HeaderMap> {
    set_cookie_header(build_refresh_token_cookie(state, "", true))
}

fn set_cookie_header(value: String) -> AppResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    let value = HeaderValue::from_str(&value)
        .map_err(|error| AppError::Internal(format!("invalid refresh token cookie: {error}")))?;
    headers.insert(header::SET_COOKIE, value);
    Ok(headers)
}

fn build_refresh_token_cookie(state: &AppState, value: &str, clear: bool) -> String {
    let max_age = if clear {
        0
    } else {
        state.refresh_token_ttl_seconds
    };
    let mut parts = vec![
        format!("{}={value}", state.refresh_token_cookie_name),
        format!("Max-Age={max_age}"),
        "Path=/api/v1/auth".to_string(),
        "HttpOnly".to_string(),
        format!("SameSite={}", state.refresh_token_cookie_same_site),
    ];

    if clear {
        parts.push("Expires=Thu, 01 Jan 1970 00:00:00 GMT".to_string());
    }
    if state.refresh_token_cookie_secure {
        parts.push("Secure".to_string());
    }

    parts.join("; ")
}
