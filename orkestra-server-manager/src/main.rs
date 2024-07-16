use std::{net::Ipv4Addr, sync::Arc};

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use handlers::{create_session, filter_sessions, join_session};
use models::SessionsInMemory;
use serde::Deserialize;
use tracing::{info, Level};
use tracing_appender::rolling;
use tracing_subscriber::{fmt::writer::MakeWriterExt, FmtSubscriber};

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
    let info_file = rolling::daily("./logs-sm", "info").with_max_level(tracing::Level::INFO);

    let subscriber = FmtSubscriber::builder()
        .with_writer(info_file)
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let config = envy::from_env::<AppConfig>()?;
    let _ = tokio::fs::create_dir("logs-ue").await;

    let context = Arc::new(Context {
        session_container: Default::default(),
        host: config.host.parse()?,
        port: config.port,
        server_path: config.server_path.clone(),
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

    pub server_path: String,
}
