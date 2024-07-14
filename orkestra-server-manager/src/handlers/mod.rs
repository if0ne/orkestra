use std::{net::SocketAddrV4, sync::Arc};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{error, info};
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
pub struct RemoveSessionRequest {
    pub server_id: Uuid,
}

pub async fn create_session(
    State(context): State<Arc<Context>>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, StatusCode> {
    let free_port = {
        let free_port = TcpListener::bind("0.0.0.0:0").await.unwrap();
        free_port.local_addr().unwrap().port()
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

    tokio::process::Command::new(&context.engine_path)
        .arg(&context.project_path)
        .arg("-server")
        .arg("-log")
        .arg(format!("-port={free_port}"))
        .arg("serverid")
        .arg(session.id.to_string())
        .arg("servercode")
        .arg(code)
        .spawn()
        .unwrap();

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
    Query(code): Query<Option<String>>,
) -> Result<Json<FilterSessionsResponse>, StatusCode> {
    info!(
        target: "Filter Session",
        event = "Fetching all game server",
    );

    let sessions: Vec<SessionPresent> = if let Some(code) = code {
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

pub async fn remove_session(
    State(context): State<Arc<Context>>,
    Json(request): Json<RemoveSessionRequest>,
) -> Result<StatusCode, StatusCode> {
    info!(
        target: "Remove Session",
        event = "Request to remove session",
        server_id = ?request.server_id
    );

    let Some(_) = context
        .session_container
        .sessions
        .remove(&request.server_id)
    else {
        error!(
            target: "Remove Session",
            event = "Not found session",
            server_id = ?request.server_id
        );

        return Err(StatusCode::BAD_REQUEST);
    };

    Ok(StatusCode::OK)
}
