use std::sync::Arc;

use axum::http::HeaderValue;
use sqlx::PgPool;

use crate::infra::s3::ObjectStorage;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cors_allowed_origins: Vec<HeaderValue>,
    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,
    pub refresh_token_ttl_seconds: i64,
    pub refresh_token_cookie_name: String,
    pub refresh_token_cookie_secure: bool,
    pub refresh_token_cookie_same_site: String,
    pub refresh_token_cookie_domain: Option<String>,
    pub s3: Option<Arc<ObjectStorage>>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: PgPool,
        cors_allowed_origins: Vec<HeaderValue>,
        jwt_secret: String,
        jwt_ttl_seconds: i64,
        refresh_token_ttl_seconds: i64,
        refresh_token_cookie_name: String,
        refresh_token_cookie_secure: bool,
        refresh_token_cookie_same_site: String,
        refresh_token_cookie_domain: Option<String>,
        s3: Option<Arc<ObjectStorage>>,
    ) -> Self {
        Self {
            db,
            cors_allowed_origins,
            jwt_secret,
            jwt_ttl_seconds,
            refresh_token_ttl_seconds,
            refresh_token_cookie_name,
            refresh_token_cookie_secure,
            refresh_token_cookie_same_site,
            refresh_token_cookie_domain,
            s3,
        }
    }
}
