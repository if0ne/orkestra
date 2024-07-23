use std::{
    net::{Ipv4Addr, SocketAddrV4},
    process::Stdio,
    sync::{atomic::AtomicU32, Arc},
};

use anyhow::Result;
use dashmap::{DashMap, DashSet};
use tokio::net::TcpListener;
use tracing::{debug, info_span, warn, Instrument};
use uuid::Uuid;

use crate::{
    models::session::{Session, SessionConfig},
    shared::config::AppConfig,
};

use super::Sesser;

static GLOBAL_CODE: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Debug)]
pub struct InMemorySesser {
    inner: Arc<InMemorySesserInner>,
}

impl InMemorySesser {
    pub fn new(config: &AppConfig) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(InMemorySesserInner {
                host: config.host.parse()?,
                project_name: config.project_name.clone(),
                sessions: Default::default(),
                pending_ports: Default::default(),
            }),
        })
    }
}

#[derive(Debug)]
struct InMemorySesserInner {
    sessions: DashMap<Uuid, Session>,
    pending_ports: DashSet<u16>,

    host: Ipv4Addr,
    project_name: String,
}

impl Sesser for InMemorySesser {
    async fn create_session(&self, config: SessionConfig) -> Result<Session> {
        let free_port = loop {
            let free_port = TcpListener::bind("0.0.0.0:0").await.unwrap();
            let free_port = free_port.local_addr().unwrap().port();

            debug!(event = "Check port for pending", port = free_port);

            if self.inner.pending_ports.insert(free_port) {
                break free_port;
            }

            debug!(event = "Port is already pending", port = free_port);
        };

        debug!(event = "Found free port", port = free_port);

        let code = GLOBAL_CODE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let code = format!("{:06}", code);

        let session = Session {
            id: Uuid::new_v4(),
            addr: SocketAddrV4::new(self.inner.host, free_port),
            title: config.title,
            code: code.clone(),
        };

        let this = self.clone();
        tokio::spawn(
            async move {
                let span = info_span!("create_game_server");

                let _guard = span.enter();

                debug!(
                    event = "Starting game server",
                    session_id = %session.id,
                    port = free_port,
                );

                let result = tokio::process::Command::new("bash")
                    .arg(format!(
                        "./{}/{}.sh",
                        this.inner.project_name, this.inner.project_name
                    ))
                    .arg("-log")
                    .arg(format!("-Port={free_port}"))
                    .arg("--serverid")
                    .arg(session.id.to_string())
                    .arg("--servercode")
                    .arg(code)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stdout(Stdio::piped())
                    .spawn();

                debug!(
                    event = "Game server started",
                    session_id = %session.id,
                    port = free_port,
                );

                let result = match result {
                    Ok(result) => result,
                    Err(err) => {
                        warn!(
                            event = "Occurs error while starting game server",
                            session_id = %session.id,
                            port = free_port,
                            error = %err
                        );
                        return;
                    }
                };

                this.inner.pending_ports.remove(&free_port);
                let result = result.wait_with_output().in_current_span().await;

                match result {
                    Ok(output) => {
                        let status = output.status;

                        if status.success() {
                            debug!(
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

                this.inner.sessions.remove(&session.id);
            }
            .in_current_span(),
        );

        debug!(
            event = "Session was saved in memory",
            session = ?session
        );

        self.inner.sessions.insert(session.id, session.clone());

        Ok(session)
    }

    fn get_by_id(&self, id: Uuid) -> Option<Session> {
        self.inner.sessions.get(&id).map(|v| v.clone())
    }

    fn get_all_sessions(&self) -> Vec<Session> {
        self.inner
            .sessions
            .iter()
            .map(|session| session.clone())
            .collect()
    }

    fn filter_by_code(&self, code: String) -> Vec<Session> {
        self.inner
            .sessions
            .iter()
            .find(|session| session.code.eq(&code))
            .map(|session| vec![session.clone().into()])
            .unwrap_or_default()
    }
}
