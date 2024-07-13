use std::{net::SocketAddrV4, sync::Arc};

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::{models::{Session, SessionPresent}, Context};

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

    let session = Session {
        id: Uuid::new_v4(),
        addr: SocketAddrV4::new(context.host, free_port),
        title: request.config.title,
    };

    let addr = session.addr.to_string();

    tokio::process::Command::new(&context.engine_path)
        .env("SERVER_ID", session.id.to_string())
        .arg(&context.project_path)
        .arg("-server")
        .arg("-log")
        .arg(format!("-port={free_port}"))
        .spawn()
        .unwrap();

    context
        .session_container
        .sessions
        .insert(session.id, session);

    Ok(Json(CreateSessionResponse { connection: addr }))
}

pub async fn filter_sessions(
    State(context): State<Arc<Context>>,
) -> Result<Json<FilterSessionsResponse>, StatusCode> {
    let sessions = context
        .session_container
        .sessions
        .iter()
        .map(|session| session.clone().into())
        .collect::<Vec<SessionPresent>>();

    Ok(Json(FilterSessionsResponse { servers: sessions }))
}

pub async fn join_session(
    State(context): State<Arc<Context>>,
    Json(request): Json<JoinSessionRequest>,
) -> Result<Json<JoinSessionResponse>, StatusCode> {
    let Some(server) = context.session_container.sessions.get(&request.server_id) else {
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
    let Some(_) = context
        .session_container
        .sessions
        .remove(&request.server_id)
    else {
        return Err(StatusCode::BAD_REQUEST);
    };

    Ok(StatusCode::OK)
}
