use std::{io, net::Ipv4Addr, sync::Arc};

use anyhow::{Ok, Result};
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

fn clone_executable(context: Arc<Context>) -> Result<()> {
    let _ = std::fs::remove_dir_all(&context.repo_name);

    let _ = std::process::Command::new("git")
        .arg("clone")
        .arg(&context.repo_path)
        .output()?;

    let _ = std::process::Command::new("git")
        .arg("lfs")
        .arg("install")
        .current_dir(&context.repo_name)
        .output()?;

    let _ = std::process::Command::new("git")
        .arg("lfs")
        .arg("fetch")
        .current_dir(&context.repo_name)
        .output()?;

    let _ = std::process::Command::new("git")
        .arg("lfs")
        .arg("pull")
        .current_dir(&context.repo_name)
        .output()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let (file_log, _guard) = {
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

    let repo_name = config.repo_path.split('/').last().map(|s| s.to_string()).unwrap();

    let context = Arc::new(Context {
        session_container: Default::default(),
        host: config.host.parse()?,
        port: config.port,
        project_name: config.project_name.clone(),
        repo_path: config.repo_path.clone(),
        repo_name: repo_name,
        used_ports: DashSet::with_capacity(16),
    });

    let app = get_router(Arc::clone(&context));
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    clone_executable(Arc::clone(&context))?;

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

    project_name: String,
    repo_path: String,
}

#[derive(Debug)]
pub struct Context {
    pub session_container: SessionsInMemory,
    pub host: Ipv4Addr,
    pub port: u16,
    pub used_ports: DashSet<u16>,

    pub project_name: String,
    pub repo_name: String,
    pub repo_path: String,
}
