use std::{net::SocketAddrV4, process::Stdio, sync::Arc};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{debug, error, info, info_span, warn, Instrument};
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
    let span = info_span!("create_session");
    let _guard = span.enter();

    info!(
        target: "create_session",
        event = "Handle request",
        request = "Create Session",
        config = ?request.config
    );

    let free_port = loop {
        let free_port = TcpListener::bind("0.0.0.0:0").await.unwrap();
        let free_port = free_port.local_addr().unwrap().port();

        debug!(
            target: "create_session",
            event = "Check port for pending",
            port = free_port
        );
        
        if context.pending_ports.insert(free_port) {
            break free_port;
        }

        debug!(
            target: "create_session",
            event = "Port is already pending",
            port = free_port
        );
    };

    info!(
        target: "create_session",
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

    let context_clone = context.clone();
    tokio::spawn(async move {
        let span = info_span!("create_game_server");

        let _guard = span.enter();

        info!(
            target: "create_game_server",
            event = "Starting game server",
            session_id = %session.id,
            port = free_port,
        );

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
            .spawn();

        info!(
            target: "create_game_server",
            event = "Game server started",
        );
        
        let result = match result {
            Ok(result) => result,
            Err(err) => {
                warn!(
                    target: "create_game_server",
                    event = "Occurs error while starting game server",
                    session_id = %session.id,
                    port = free_port,
                    error = %err
                );
                return;
            }
        };

        context_clone.pending_ports.remove(&free_port);
        let result = result.wait_with_output().in_current_span().await;

        match result {
            Ok(output) => {
                let status = output.status;

                if status.success() {
                    info!(
                        target: "game_server",
                        event = "Game server was finished and removed",
                        session_id = ?session.id,
                        port = free_port,
                    );
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    warn!(
                        target: "game_server",
                        event = "Game server exit with error",
                        session_id = %session.id,
                        port = free_port,
                        status = %status,
                        stderr = %stderr
                    );
                }
            }
            Err(err) => {
                warn!(
                    target: "game_server",
                    event = "Occurs error while running the game server",
                    session_id = %session.id,
                    port = free_port,
                    error = %err
                );
            }
        }

        context_clone.session_container.sessions.remove(&session.id);
    }.in_current_span());

    info!(
        target: "create_session",
        event = "Session was saved in memory",
        session = ?session
    );

    context
        .session_container
        .sessions
        .insert(session.id, session);

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
