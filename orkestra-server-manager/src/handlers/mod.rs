use std::{net::SocketAddrV4, process::Stdio, sync::Arc};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    models::{Session, SessionPresent, GLOBAL_CODE},
    Context,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSessionRequest {
    pub config: SessionConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionConfig {
    pub max_players: u32,
    pub game_map: String,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSessionResponse {
    pub connection: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JoinSessionRequest {
    pub server_id: Uuid,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JoinSessionResponse {
    pub connection: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilterSessionsResponse {
    pub servers: Vec<SessionPresent>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilterParams {
    pub code: Option<String>,
}

pub async fn create_session(
    State(context): State<Arc<Context>>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, StatusCode> {
    let free_port = loop {
        let free_port = TcpListener::bind("0.0.0.0:0").await.unwrap();
        let free_port = free_port.local_addr().unwrap().port();

        if context.used_ports.insert(free_port) {
            break free_port;
        }
    };

    info!(
        target: "Create Session",
        event = "Found free port",
        port = free_port
    );

    let code = GLOBAL_CODE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let code = format!("{:06}", code);

    let session = Session {
        id: Uuid::new_v4(),
        addr: SocketAddrV4::new(context.host, free_port),
        title: request.config.title,
        code: code.clone(),
    };

    let addr = session.addr.to_string();

    info!(
        target: "Create Session",
        event = "Starting game server",
    );

    let context_clone = context.clone();
    tokio::spawn(async move {
        let result = tokio::process::Command::new("bash")
            .arg(format!("./{}/{}.sh", context_clone.project_name, context_clone.project_name))
            .arg("-log")
            .arg(format!("-Port={free_port}"))
            .arg("--serverid")
            .arg(session.id.to_string())
            .arg("--servercode")
            .arg(code)
            .arg("--serveraddr")
            .arg(format!("{}:{}", context_clone.host, context_clone.port))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .output()
            .await;

        match result {
            Ok(output) => {
                // let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let status = output.status.to_string();

                info!(
                    target: "Clear Session",
                    event = "Game server was finished and removed",
                    // stdout = stdout,
                    stderr = stderr,
                    status = status
                );
            }
            Err(err) => {
                warn!(
                    target: "Create Session",
                    event = "Game server end with error",
                    error = ?err
                );
            }
        }

        context_clone.session_container.sessions.remove(&session.id);
        context_clone.used_ports.remove(&free_port);
    });

    context
        .session_container
        .sessions
        .insert(session.id, session);

    info!(
        target: "Create Session",
        event = "Game server was saved",
    );

    Ok(Json(CreateSessionResponse { connection: addr }))
}

pub async fn filter_sessions(
    State(context): State<Arc<Context>>,
    Query(request): Query<FilterParams>,
) -> Result<Json<FilterSessionsResponse>, StatusCode> {
    info!(
        target: "Filter Session",
        event = "Fetching all game server",
    );

    let sessions: Vec<SessionPresent> = if let Some(code) = request.code {
        context
            .session_container
            .sessions
            .iter()
            .find(|session| session.code.eq(&code))
            .map(|session| vec![session.clone().into()])
            .unwrap_or_default()
    } else {
        context
            .session_container
            .sessions
            .iter()
            .map(|session| session.clone().into())
            .collect()
    };

    Ok(Json(FilterSessionsResponse { servers: sessions }))
}

pub async fn join_session(
    State(context): State<Arc<Context>>,
    Json(request): Json<JoinSessionRequest>,
) -> Result<Json<JoinSessionResponse>, StatusCode> {
    info!(
        target: "Join Session",
        event = "Request to join session",
        server_id = ?request.server_id
    );

    let Some(server) = context.session_container.sessions.get(&request.server_id) else {
        error!(
            target: "Join Session",
            event = "Not found session",
            server_id = ?request.server_id
        );

        return Err(StatusCode::BAD_REQUEST);
    };

    Ok(Json(JoinSessionResponse {
        connection: server.addr.to_string(),
    }))
}
