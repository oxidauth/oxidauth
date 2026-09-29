use sqlx::PgPool;

use super::PgError;

#[tracing::instrument(name = "ping", level = "info")]
pub async fn ping(pool: &PgPool) -> Result<(), PgError> {
    sqlx::query(include_str!("./ping.sql"))
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn it_should_ping_successfully(pool: PgPool) {
        let result = ping(&pool).await;

        assert!(result.is_ok());
    }
}
