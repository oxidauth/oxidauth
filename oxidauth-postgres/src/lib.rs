pub mod prelude;
pub mod queries;
pub mod rows;

use std::env::var;

use sqlx::{postgres::PgPoolOptions, PgPool};

const DATABASE_HOSTNAME: &str = "DATABASE_HOSTNAME";
const DATABASE_PORT: &str = "DATABASE_PORT";
const DATABASE_NAME: &str = "DATABASE_NAME";
const DATABASE_USERNAME: &str = "DATABASE_USERNAME";
const DATABASE_PASSWORD: &str = "DATABASE_PASSWORD";
const MIGRATIONS_ENABLED: &str = "MIGRATIONS_ENABLED";

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(url: impl Into<String>) -> Result<Self, OxidPgError> {
        let url = url.into();

        let pool = PgPoolOptions::new().connect(&url).await?;

        let database = Database { pool };

        Ok(database)
    }

    pub async fn from_env() -> Result<Self, OxidPgError> {
        let host = from_env(DATABASE_HOSTNAME)?;
        let port = from_env(DATABASE_PORT)?;
        let database = from_env(DATABASE_NAME)?;
        let username = from_env(DATABASE_USERNAME)?;
        let password = from_env(DATABASE_PASSWORD)?;

        let url = format!("postgres://{username}:{password}@{host}:{port}/{database}");

        Self::new(url).await
    }

    pub async fn migrate(&self) -> Result<(), OxidPgError> {
        let migrations_enabled = from_env(MIGRATIONS_ENABLED)?;

        if migrations_enabled != "true" {
            // info!("migrations are not enabled");

            return Ok(());
        }

        // info!("migrating...");

        // sqlx::migrate!().run(&self.pool).await?;

        // info!("migration finished!");

        Ok(())
    }
}

fn from_env(env: &'static str) -> Result<String, OxidPgError> {
    let variable = var(env).map_err(|_| OxidPgError::MissingEnvVar(env))?;

    Ok(variable)
}

pub enum OxidPgError {
    MissingEnvVar(&'static str),
    Sqlx(sqlx::Error),
    NotFound,
}

impl From<sqlx::Error> for OxidPgError {
    fn from(err: sqlx::Error) -> Self {
        Self::Sqlx(err)
    }
}
