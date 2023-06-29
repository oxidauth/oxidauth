use sqlx::PgPool;

pub mod authorities;
pub mod prelude;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(Self { pool })
    }

    pub async fn ping(&self) -> Result<(), sqlx::Error> {
        sqlx::query_as::<_, (i64,)>("SELECT (1)")
            .fetch_one(&self.pool)
            .await?;

        Ok(())
    }
}
