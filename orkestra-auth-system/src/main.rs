use std::{future::IntoFuture, io, sync::Arc};

use anyhow::Result;
use axum::{routing::post, Router};
use handlers::{login, signup};
use rand::{rngs::OsRng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tokio::sync::Mutex;
use tracing::{info, info_span, Instrument};
use tracing_appender::rolling;
use tracing_subscriber::{
    fmt::{self, writer::MakeWriterExt},
    layer::SubscriberExt,
};

mod handlers;

async fn create_database_connection(config: &AppConfig) -> Result<Pool<Postgres>> {
    info!(
        target: "database",
        event = "Connecting to PostgresSQL",
    );

    let conn = PgPoolOptions::new()
        .max_connections(20)
        .connect(&format!(
            "postgres://{}:{}@{}:{}/postgres",
            config.database_username,
            config.database_password,
            config.database_host,
            config.database_port
        ))
        .await?;

    info!(
        target: "database",
        event = "Successfully connected to database",
    );

    Ok(conn)
}

fn get_router(context: Arc<Context>) -> Router {
    Router::new()
        .route("/auth/v1/signup", post(signup))
        .route("/auth/v1/login", post(login))
        .with_state(context)
}

#[tokio::main]
async fn main() -> Result<()> {
    let span = info_span!("auth_system");
    let _guard = span.enter();

    let (file_log, _guard) = {
        let (file_log, guard) = tracing_appender::non_blocking(rolling::daily("./logs-as", "info"));

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

    let config = envy::from_env::<AppConfig>().unwrap();

    let database_connection = create_database_connection(&config)
        .in_current_span()
        .await?;

    let context = Arc::new(Context {
        database_connection,
        random: Mutex::new(ChaCha8Rng::seed_from_u64(OsRng.next_u64())),
    });

    info!(
        target: "database",
        event = "Running migrations",
    );

    sqlx::migrate!()
        .run(&context.database_connection)
        .in_current_span()
        .await?;

    info!(
        target: "database",
        event = "Migrated",
    );

    let app = get_router(context);
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!(
        target: "auth_system",
        event = "Start listening", addr = addr
    );

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = axum::serve(listener, app).into_future() => {},
    }

    info!(
        target: "auth_system",
        event = "Shutdown the server",
    );

    Ok(())
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
struct AppConfig {
    database_host: String,
    database_port: u16,
    database_username: String,
    database_password: String,

    port: u16,
}

pub struct Context {
    pub database_connection: Pool<Postgres>,
    pub random: Mutex<ChaCha8Rng>,
}
