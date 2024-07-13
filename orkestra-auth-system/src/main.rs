use std::sync::Arc;

use anyhow::Result;
use axum::{routing::post, Router};
use handlers::{login, signup};
use rand::{rngs::OsRng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tokio::sync::Mutex;

mod handlers;

async fn create_database_connection(config: &AppConfig) -> Result<Pool<Postgres>> {
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
    let config = envy::from_env::<AppConfig>().unwrap();

    let database_connection = create_database_connection(&config).await?;

    let mut os_rng = OsRng::default();

    let context = Arc::new(Context {
        database_connection,
        random: Mutex::new(ChaCha8Rng::seed_from_u64(os_rng.next_u64())),
    });

    sqlx::migrate!().run(&context.database_connection).await?;

    let app = get_router(context);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;

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
