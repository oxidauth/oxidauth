use sqlx::PgPool;
use std::error::Error;

pub mod authorities;
pub mod prelude;
pub mod users;

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
            "postgres://oxidauth:oxidauth@127.0.0.1:5432/oxidauth",
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
}
