use anyhow::Result;
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod handlers;
mod models;
mod services;

#[tokio::main]
async fn main() -> Result<()> {
    let config = envy::from_env::<AppConfig>().unwrap();

    let context = Context {
        database_connection: PgPoolOptions::new()
            .max_connections(20)
            .connect(&format!(
                "postgres://{}:{}@{}:{}/postgres",
                config.database_username,
                config.database_password,
                config.database_host,
                config.database_port
            ))
            .await
            .unwrap(),
    };

    sqlx::migrate!().run(&context.database_connection).await?;

    println!("connected and migrate");

    Ok(())
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
struct AppConfig {
    database_host: String,
    database_port: u16,
    database_username: String,
    database_password: String,
}

pub struct Context {
    pub database_connection: Pool<Postgres>,
}
