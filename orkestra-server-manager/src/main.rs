use std::{io, net::Ipv4Addr, sync::Arc};

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use dashmap::DashSet;
use handlers::{create_session, filter_sessions, join_session};
use models::SessionsInMemory;
use serde::Deserialize;
use tracing::info;
use tracing_appender::rolling;
use tracing_subscriber::{
    fmt::{self, writer::MakeWriterExt},
    layer::SubscriberExt,
};

mod handlers;
mod models;

fn get_router(context: Arc<Context>) -> Router {
    Router::new()
        .route("/api/v1/create", post(create_session))
        .route("/api/v1/filter", get(filter_sessions))
        .route("/api/v1/join", post(join_session))
        .with_state(context)
}

#[tokio::main]
async fn main() -> Result<()> {
    let (file_log, guard) = {
        let (file_log, guard) = tracing_appender::non_blocking(rolling::daily("./logs-sm", "info"));

        let file_log = fmt::Layer::new()
            .with_ansi(false)
            .with_writer(file_log.with_max_level(tracing::Level::INFO));

        (file_log, guard)
    };

    let console_log = fmt::Layer::new().with_ansi(true).with_writer(io::stdout);

    let subscriber = tracing_subscriber::registry()
        .with(file_log)
        .with(console_log);

    let _ = tracing::subscriber::set_global_default(subscriber);

    let config = envy::from_env::<AppConfig>()?;
    let _ = tokio::fs::create_dir("logs-ue").await;

    let context = Arc::new(Context {
        session_container: Default::default(),
        host: config.host.parse()?,
        port: config.port,
        server_path: config.server_path.clone(),
        used_ports: DashSet::with_capacity(16),
    });

    let app = get_router(context);
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!(
        target: "Server",
        event = "Start listening",
        addr = addr
    );
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
struct AppConfig {
    host: String,
    port: u16,

    server_path: String,
}

#[derive(Debug)]
pub struct Context {
    pub session_container: SessionsInMemory,
    pub host: Ipv4Addr,
    pub port: u16,
    pub used_ports: DashSet<u16>,

    pub server_path: String,
}
