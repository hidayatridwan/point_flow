use std::time::Duration;

use anyhow::Result;
use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn connect(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        // Recycle connections periodically so the pool re-resolves DNS:
        // a persistent connection pinned to a bad-slow ELB IP must not
        // poison every query forever (observed ~70ms RTT per statement).
        .idle_timeout(Duration::from_secs(60))
        .max_lifetime(Duration::from_secs(600))
        .connect(database_url)
        .await?;

    Ok(pool)
}
