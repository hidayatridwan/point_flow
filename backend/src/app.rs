use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,
    pub refresh_token_ttl_seconds: i64,
    pub refresh_token_cookie_name: String,
    pub refresh_token_cookie_secure: bool,
    pub refresh_token_cookie_same_site: String,
}

impl AppState {
    pub fn new(
        db: PgPool,
        jwt_secret: String,
        jwt_ttl_seconds: i64,
        refresh_token_ttl_seconds: i64,
        refresh_token_cookie_name: String,
        refresh_token_cookie_secure: bool,
        refresh_token_cookie_same_site: String,
    ) -> Self {
        Self {
            db,
            jwt_secret,
            jwt_ttl_seconds,
            refresh_token_ttl_seconds,
            refresh_token_cookie_name,
            refresh_token_cookie_secure,
            refresh_token_cookie_same_site,
        }
    }
}
