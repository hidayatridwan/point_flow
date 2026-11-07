use std::{env, net::IpAddr};

use anyhow::{Context, Result, bail};
use axum::http::HeaderValue;

pub struct S3Settings {
    pub endpoint_url: String,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub public_url: String,
    pub force_path_style: bool,
}

pub struct AppConfig {
    pub host: IpAddr,
    pub port: u16,
    pub database_url: String,
    pub cors_allowed_origins: Vec<HeaderValue>,
    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,
    pub refresh_token_ttl_seconds: i64,
    pub refresh_token_cookie_name: String,
    pub refresh_token_cookie_secure: bool,
    pub refresh_token_cookie_same_site: String,
    pub refresh_token_cookie_domain: Option<String>,
    pub s3: Option<S3Settings>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let host = env::var("APP_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string())
            .parse()
            .context("APP_HOST must be a valid IP address")?;
        let port = env::var("APP_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .context("APP_PORT must be a valid TCP port")?;
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
        let cors_allowed_origins =
            parse_cors_allowed_origins(&env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default())?;
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            "dev-only-change-this-secret-before-running-in-production".to_string()
        });
        let jwt_ttl_seconds = env::var("JWT_TTL_SECONDS")
            .unwrap_or_else(|_| "900".to_string())
            .parse()
            .context("JWT_TTL_SECONDS must be an integer")?;
        let refresh_token_ttl_seconds = env::var("REFRESH_TOKEN_TTL_SECONDS")
            .unwrap_or_else(|_| "2592000".to_string())
            .parse()
            .context("REFRESH_TOKEN_TTL_SECONDS must be an integer")?;
        let refresh_token_cookie_name =
            env::var("REFRESH_TOKEN_COOKIE_NAME").unwrap_or_else(|_| "refresh_token".to_string());
        let refresh_token_cookie_secure: bool = env::var("REFRESH_TOKEN_COOKIE_SECURE")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .context("REFRESH_TOKEN_COOKIE_SECURE must be a boolean")?;
        let refresh_token_cookie_same_site = normalize_same_site(
            &env::var("REFRESH_TOKEN_COOKIE_SAME_SITE").unwrap_or_else(|_| "Lax".to_string()),
        )?;
        let refresh_token_cookie_domain =
            normalize_cookie_domain(env::var("REFRESH_TOKEN_COOKIE_DOMAIN").ok())?;

        let s3 = load_s3_settings()?;

        if jwt_ttl_seconds <= 0 {
            bail!("JWT_TTL_SECONDS must be greater than zero");
        }
        if refresh_token_ttl_seconds <= 0 {
            bail!("REFRESH_TOKEN_TTL_SECONDS must be greater than zero");
        }
        if refresh_token_cookie_name.trim().is_empty() {
            bail!("REFRESH_TOKEN_COOKIE_NAME must not be empty");
        }
        if refresh_token_cookie_same_site == "None" && !refresh_token_cookie_secure {
            bail!("REFRESH_TOKEN_COOKIE_SAME_SITE=None requires REFRESH_TOKEN_COOKIE_SECURE=true");
        }

        Ok(Self {
            host,
            port,
            database_url,
            cors_allowed_origins,
            jwt_secret,
            jwt_ttl_seconds,
            refresh_token_ttl_seconds,
            refresh_token_cookie_name,
            refresh_token_cookie_secure,
            refresh_token_cookie_same_site,
            refresh_token_cookie_domain,
            s3,
        })
    }
}

fn load_s3_settings() -> Result<Option<S3Settings>> {
    let endpoint_url = env::var("S3_ENDPOINT_URL").unwrap_or_default();
    let bucket = env::var("S3_BUCKET").unwrap_or_default();
    let access_key_id = env::var("S3_ACCESS_KEY_ID").unwrap_or_default();
    let secret_access_key = env::var("S3_SECRET_ACCESS_KEY").unwrap_or_default();

    if endpoint_url.trim().is_empty()
        || bucket.trim().is_empty()
        || access_key_id.trim().is_empty()
        || secret_access_key.trim().is_empty()
    {
        return Ok(None);
    }

    let region = env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let public_url = env::var("S3_PUBLIC_URL").unwrap_or_default();
    let public_url = if public_url.trim().is_empty() {
        format!("{}/{}", endpoint_url.trim_end_matches('/'), bucket.trim())
    } else {
        public_url
    };
    let force_path_style: bool = env::var("S3_FORCE_PATH_STYLE")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .context("S3_FORCE_PATH_STYLE must be a boolean")?;

    Ok(Some(S3Settings {
        endpoint_url,
        region,
        bucket,
        access_key_id,
        secret_access_key,
        public_url,
        force_path_style,
    }))
}

fn parse_cors_allowed_origins(value: &str) -> Result<Vec<HeaderValue>> {
    value
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(parse_cors_origin)
        .collect()
}

fn parse_cors_origin(origin: &str) -> Result<HeaderValue> {
    if origin == "*" {
        bail!("CORS_ALLOWED_ORIGINS does not allow wildcard '*' when credentials are enabled");
    }
    if !(origin.starts_with("http://") || origin.starts_with("https://")) {
        bail!("CORS_ALLOWED_ORIGINS entries must include http:// or https://");
    }
    if origin.ends_with('/') {
        bail!("CORS_ALLOWED_ORIGINS entries must not end with a trailing slash");
    }

    HeaderValue::from_str(origin)
        .with_context(|| format!("CORS_ALLOWED_ORIGINS contains an invalid origin: {origin}"))
}

fn normalize_same_site(value: &str) -> Result<String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "lax" => Ok("Lax".to_string()),
        "strict" => Ok("Strict".to_string()),
        "none" => Ok("None".to_string()),
        _ => bail!("REFRESH_TOKEN_COOKIE_SAME_SITE must be Lax, Strict, or None"),
    }
}

fn normalize_cookie_domain(value: Option<String>) -> Result<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let domain = value.trim();
    if domain.is_empty() {
        return Ok(None);
    }
    if domain.starts_with("http://") || domain.starts_with("https://") {
        bail!("REFRESH_TOKEN_COOKIE_DOMAIN must be a domain, not a URL");
    }
    if domain.contains(';') || domain.contains(',') || domain.chars().any(char::is_whitespace) {
        bail!("REFRESH_TOKEN_COOKIE_DOMAIN contains invalid characters");
    }

    Ok(Some(domain.to_string()))
}
