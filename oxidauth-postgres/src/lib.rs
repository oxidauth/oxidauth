use sqlx::PgPool;
use std::{env, error::Error};

pub mod auth;
pub mod authorities;
pub mod invitations;
pub mod permissions;
pub mod private_keys;
pub mod public_keys;
pub mod refresh_tokens;
pub mod role_permission_grants;
pub mod role_role_grants;
pub mod roles;
pub mod settings;
pub mod user_authorities;
pub mod user_permission_grants;
pub mod user_role_grants;
pub mod users;

pub mod prelude;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    #[tracing::instrument(name = "creating oxidauth db", level = "trace")]
    pub fn new(
        pool: PgPool,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        Ok(Self { pool })
    }

    #[tracing::instrument(
        name = "creating oxidauth db from env",
        level = "trace"
    )]
    pub async fn from_env(
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let database_url = env::var("DATABASE_URL")?;

        let pool = PgPool::connect(&database_url).await?;

        Self::new(pool)
    }

    pub async fn ping(&self) -> Result<(), sqlx::Error> {
        sqlx::query_as::<_, (i32,)>("SELECT (1)")
            .fetch_one(&self.pool)
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!()
            .run(&self.pool)
            .await?;

        Ok(())
    }
}
