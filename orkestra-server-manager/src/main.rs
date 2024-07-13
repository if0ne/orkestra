use std::{net::Ipv4Addr, sync::Arc};

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use handlers::{create_session, filter_sessions, join_session, remove_session};
use models::SessionsInMemory;
use serde::Deserialize;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod handlers;
mod models;

fn get_router(context: Arc<Context>) -> Router {
    Router::new()
        .route("/api/v1/create", post(create_session))
        .route("/api/v1/filter", get(filter_sessions))
        .route("/api/v1/join", post(join_session))
        .route("/api/v1/remove", post(remove_session))
        .with_state(context)
}

#[tokio::main]
async fn main() -> Result<()> {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());

    let subscriber = FmtSubscriber::builder()
        .with_writer(non_blocking)
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = envy::from_env::<AppConfig>().unwrap();

    let context = Arc::new(Context {
        session_container: Default::default(),
        host: config.host.parse().unwrap(),
        engine_path: config.engine_path.clone(),
        project_path: config.project_path.clone(),
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

    engine_path: String,
    project_path: String,
}

#[derive(Debug)]
pub struct Context {
    pub session_container: SessionsInMemory,
    pub host: Ipv4Addr,

    pub engine_path: String,
    pub project_path: String,
}
