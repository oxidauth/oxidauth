use crate::prelude::*;

impl Database {
    async fn all_users(&self) -> Result<Vec<UserRow>> {
        let mut conn = self.pool.acquire().await?;

        all_users_query(&mut conn).await
    }
}

pub async fn all_users_query(conn: &mut PgConnection) -> Result<Vec<UserRow>> {
    let rows = sqlx::query_as::<_, UserRow>(include_str!("./all_users.sql"))
        .fetch_all(conn)
        .await?;

    Ok(rows)
}
