use std::{future::IntoFuture, net::SocketAddr};

use anyhow::{Ok, Result};

use shared::{
    config::AppConfig,
    context::Context,
    logger::Logger,
    router::v1,
    services::{
        server_cloner::{simple_server_cloner::SimplerServerCloner, ServerCloner},
        sesser::inmemory_sesser::InMemorySesser,
    },
};
use tracing::{info, info_span};

mod models;
mod plugins;
mod shared;

#[tokio::main]
async fn main() -> Result<()> {
    let span = info_span!("server_manager");
    let _guard = span.enter();

    let _logger = Logger::new();
    let config = AppConfig::load()?;

    let sesser = InMemorySesser::new(&config)?;
    let context = Context::new(&config, sesser)?;

    let server_cloner = SimplerServerCloner::new(context.clone());
    server_cloner.clone_server_repo()?;

    let v1 = v1(context);
    let app = v1;

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!(event = "Start listening", addr = addr);

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).into_future() => {},
    }

    let span = info_span!("server_manager");
    let _guard = span.enter();

    info!(event = "Shutdown the server",);

    Ok(())
}
