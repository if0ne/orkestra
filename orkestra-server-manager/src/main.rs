use std::{future::IntoFuture, io, net::Ipv4Addr, sync::Arc};

use anyhow::{Ok, Result};
use axum::{
    routing::{get, post},
    Router,
};
use dashmap::DashSet;
use handlers::{create_session, filter_sessions, join_session};
use models::SessionsInMemory;
use serde::Deserialize;
use tracing::{debug, info, info_span};
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
    let span = info_span!("clone_server_repo");
    let _guard = span.enter();

    info!(event = "Start cloning server repository",);

    debug!(event = "Clean up old version of server repository",);

    let _ = std::fs::remove_dir_all(&context.project_name);

    debug!(event = "Clone server repository",);

    let _ = std::process::Command::new("git")
        .arg("clone")
        .arg(&context.repo_path)
        .output()?;

    debug!(event = "Fetch lfs files",);

    let _ = std::process::Command::new("git")
        .arg("lfs")
        .arg("fetch")
        .current_dir(&context.project_name)
        .output()?;

    debug!(event = "Pull lfs files",);

    let _ = std::process::Command::new("git")
        .arg("lfs")
        .arg("pull")
        .current_dir(&context.project_name)
        .output()?;

    info!(event = "Server repository was successfully downloaded",);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let span = info_span!("server_manager");
    let _guard = span.enter();

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

    let context = Arc::new(Context {
        session_container: Default::default(),
        host: config.host.parse()?,
        port: config.port,
        project_name: config.project_name.clone(),
        repo_path: config.repo_path.clone(),
        pending_ports: DashSet::with_capacity(16),
    });

    let app = get_router(Arc::clone(&context));
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    clone_executable(Arc::clone(&context))?;

    info!(event = "Start listening", addr = addr);

    drop(_guard);

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = axum::serve(listener, app).into_future() => {},
    }

    let span = info_span!("server_manager");
    let _guard = span.enter();

    info!(event = "Shutdown the server",);

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
    pub pending_ports: DashSet<u16>,

    pub project_name: String,
    pub repo_path: String,
}
