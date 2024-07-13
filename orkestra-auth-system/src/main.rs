use std::sync::Arc;

use anyhow::Result;
use axum::{routing::post, Router};
use handlers::{login, signup};
use rand::{rngs::OsRng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tokio::sync::Mutex;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod handlers;

async fn create_database_connection(config: &AppConfig) -> Result<Pool<Postgres>> {
    info!(
        target: "Database",
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
        target: "Database",
        event = "Successfully connected",
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
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());

    let subscriber = FmtSubscriber::builder()
        .with_writer(non_blocking)
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = envy::from_env::<AppConfig>().unwrap();

    let database_connection = create_database_connection(&config).await?;

    let mut os_rng = OsRng::default();

    let context = Arc::new(Context {
        database_connection,
        random: Mutex::new(ChaCha8Rng::seed_from_u64(os_rng.next_u64())),
    });

    info!(
        target: "Database",
        event = "Running migrations",
    );

    sqlx::migrate!().run(&context.database_connection).await?;

    info!(
        target: "Database",
        event = "Migrated",
    );

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
