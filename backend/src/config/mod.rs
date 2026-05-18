use std::{env, net::IpAddr};

use anyhow::{Context, Result, bail};

pub struct AppConfig {
    pub host: IpAddr,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,
    pub refresh_token_ttl_seconds: i64,
    pub refresh_token_cookie_name: String,
    pub refresh_token_cookie_secure: bool,
    pub refresh_token_cookie_same_site: String,
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
            jwt_secret,
            jwt_ttl_seconds,
            refresh_token_ttl_seconds,
            refresh_token_cookie_name,
            refresh_token_cookie_secure,
            refresh_token_cookie_same_site,
        })
    }
}

fn normalize_same_site(value: &str) -> Result<String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "lax" => Ok("Lax".to_string()),
        "strict" => Ok("Strict".to_string()),
        "none" => Ok("None".to_string()),
        _ => bail!("REFRESH_TOKEN_COOKIE_SAME_SITE must be Lax, Strict, or None"),
    }
}
