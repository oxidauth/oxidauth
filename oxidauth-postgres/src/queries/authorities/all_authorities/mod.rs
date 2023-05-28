use crate::{prelude::*, rows::authority::AuthorityRow};

impl Database {
    async fn all_authorities(&self) -> Result<Vec<AuthorityRow>> {
        let mut conn = self.pool.acquire().await?;

        all_authorities_query(&mut conn).await
    }
}

pub async fn all_authorities_query(conn: &mut PgConnection) -> Result<Vec<AuthorityRow>> {
    let row = sqlx::query_as::<_, AuthorityRow>(include_str!("./all_authorities.sql"))
        .fetch_all(conn)
        .await?;

    Ok(row)
}
