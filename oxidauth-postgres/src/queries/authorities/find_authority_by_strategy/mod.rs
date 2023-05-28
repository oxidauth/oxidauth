use crate::{prelude::*, rows::authority::AuthorityRow};

impl Database {
    async fn find_authority_by_strategy(&self, strategy: String) -> Result<Vec<AuthorityRow>> {
        let mut conn = self.pool.acquire().await?;

        find_authority_by_strategy_query(&mut conn, strategy).await
    }
}

pub async fn find_authority_by_strategy_query(
    conn: &mut PgConnection,
    strategy: String,
) -> Result<Vec<AuthorityRow>> {
    let row = sqlx::query_as::<_, AuthorityRow>(include_str!("./find_authority_by_strategy.sql"))
        .bind(strategy)
        .fetch_all(conn)
        .await?;

    Ok(row)
}
