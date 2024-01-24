use sqlx::PgPool;
use std::error::Error;

pub mod auth;
pub mod authorities;
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
    pub fn new(
        pool: PgPool,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        Ok(Self { pool })
    }

    pub async fn from_env(
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let pool = PgPool::connect(
            "postgres://oxidauth:oxidauth@postgres.oxidauth.test:5432/oxidauth",
        )
        .await?;

        Self::new(pool)
    }

    pub async fn ping(&self) -> Result<(), sqlx::Error> {
        sqlx::query_as::<_, (i64,)>("SELECT (1)")
            .fetch_one(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!()
            .run(&self.pool)
            .await?;

        Ok(())
    }
}
