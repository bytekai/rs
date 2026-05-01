mod error;
mod task;

use sqlx::sqlite::SqlitePoolOptions;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_tree::HierarchicalLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .init();

    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://app.db?mode=rwc".into());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = task::routes().with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!(addr = "0.0.0.0:3000", "rs ready");
    axum::serve(listener, app).await?;
    Ok(())
}
