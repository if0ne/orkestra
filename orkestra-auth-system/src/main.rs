use std::future::IntoFuture;

use anyhow::Result;
use shared::{config::AppConfig, context::Context, database::Database, logger::Logger, router::v1};
use tracing::{info, info_span, Instrument};

mod plugins;
mod shared;

#[tokio::main]
async fn main() -> Result<()> {
    let span = info_span!("auth_system");
    let _guard = span.enter();

    let _logger = Logger::new();

    let config = AppConfig::load()?;

    let database = Database::new(&config).in_current_span().await?;
    database.migrate().in_current_span().await?;

    let context = Context::new(database);

    let app = v1(context);
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!(
        target: "auth_system",
        event = "Start listening",
        addr = addr
    );

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = axum::serve(listener, app).into_future() => {},
    }

    info!(
        target: "auth_system",
        event = "Shutdown the server",
    );

    Ok(())
}
