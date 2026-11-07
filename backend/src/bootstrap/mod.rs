use std::net::SocketAddr;

use anyhow::Context;
use axum::{Router, http::HeaderValue, middleware};
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::{
    app::AppState,
    common::request_context,
    config::AppConfig,
    http,
    infra::{db, s3::ObjectStorage},
};

pub async fn run() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = AppConfig::from_env().context("failed to load application configuration")?;
    let pool = db::connect(&config.database_url)
        .await
        .context("failed to connect to database")?;

    let s3 = match config.s3 {
        Some(ref settings) => {
            let storage = ObjectStorage::from_settings(settings)
                .await
                .context("failed to initialize object storage")?;
            tracing::info!(bucket = %settings.bucket, "object storage enabled");
            Some(std::sync::Arc::new(storage))
        }
        None => {
            tracing::warn!("S3_* env not configured; file uploads are disabled");
            None
        }
    };

    let state = AppState::new(
        pool,
        config.cors_allowed_origins,
        config.jwt_secret,
        config.jwt_ttl_seconds,
        config.refresh_token_ttl_seconds,
        config.refresh_token_cookie_name,
        config.refresh_token_cookie_secure,
        config.refresh_token_cookie_same_site,
        config.refresh_token_cookie_domain,
        s3,
    );
    let addr = SocketAddr::from((config.host, config.port));
    let app = app(state);

    tracing::info!(%addr, "starting backend API");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

pub fn app(state: AppState) -> Router {
    let cors = cors_layer(&state.cors_allowed_origins);

    Router::new()
        .nest("/api/v1", http::router())
        .nest("/api/v1", http::public_router())
        .layer(CompressionLayer::new())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(middleware::from_fn(request_context::bind_request_id))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .with_state(state)
}

fn cors_layer(allowed_origins: &[HeaderValue]) -> CorsLayer {
    let allow_origin = if allowed_origins.is_empty() {
        AllowOrigin::mirror_request()
    } else {
        AllowOrigin::list(allowed_origins.to_vec())
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods(AllowMethods::mirror_request())
        .allow_headers(AllowHeaders::mirror_request())
        .allow_credentials(true)
}

fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
