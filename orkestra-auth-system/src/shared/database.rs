use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tracing::{info, Instrument};

use super::config::AppConfig;

#[derive(Clone)]
pub struct Database {
    inner: Pool<Postgres>,
}

impl Database {
    pub async fn new(config: &AppConfig) -> Result<Self> {
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
            .in_current_span()
            .await?;

        info!(
            target: "database",
            event = "Successfully connected to database",
        );

        Ok(Self { inner: conn })
    }

    pub async fn migrate(&self) -> Result<()> {
        info!(
            target: "database",
            event = "Running migrations",
        );

        sqlx::migrate!().run(&self.inner).in_current_span().await?;

        info!(
            target: "database",
            event = "Migrated",
        );

        Ok(())
    }
}

impl AsRef<Pool<Postgres>> for Database {
    fn as_ref(&self) -> &Pool<Postgres> {
        &self.inner
    }
}
