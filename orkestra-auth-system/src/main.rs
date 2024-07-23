use std::{future::IntoFuture, net::SocketAddr};

use anyhow::Result;
use shared::{
    config::AppConfig,
    context::Context,
    database::Database,
    integrations::vk::{api::VkService, router::vk_integration},
    logger::Logger,
    router::v1,
};
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

    let vk_service = VkService::new(&config.vk_game_id, &config.vk_gas_secret);
    let vk_integration = vk_integration(vk_service);

    let v1 = v1(context);

    let app = v1.merge(vk_integration);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!(
        target: "auth_system",
        event = "Start listening",
        addr = addr
    );

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).into_future() => {},
    }

    info!(
        target: "auth_system",
        event = "Shutdown the server",
    );

    Ok(())
}
