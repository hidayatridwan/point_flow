use std::net::SocketAddr;

use anyhow::Context;
use axum::{Router, middleware};
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::{app::AppState, common::request_context, config::AppConfig, http, infra::db};

pub async fn run() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = AppConfig::from_env().context("failed to load application configuration")?;
    let pool = db::connect(&config.database_url)
        .await
        .context("failed to connect to database")?;

    let state = AppState::new(
        pool,
        config.jwt_secret,
        config.jwt_ttl_seconds,
        config.refresh_token_ttl_seconds,
        config.refresh_token_cookie_name,
        config.refresh_token_cookie_secure,
        config.refresh_token_cookie_same_site,
    );
    let addr = SocketAddr::from((config.host, config.port));
    let app = app(state);

    tracing::info!(%addr, "starting backend API");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", http::router())
        .layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::mirror_request())
                .allow_methods(AllowMethods::mirror_request())
                .allow_headers(AllowHeaders::mirror_request())
                .allow_credentials(true),
        )
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(middleware::from_fn(request_context::bind_request_id))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .with_state(state)
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
