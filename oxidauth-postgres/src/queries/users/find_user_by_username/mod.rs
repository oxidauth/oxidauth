use crate::prelude::*;

impl Database {
    pub async fn find_user_by_username(&self, username: &str) -> Result<UserRow> {
        let mut conn = self.pool.acquire().await?;

        find_user_by_username_query(&mut conn, username).await
    }
}

pub async fn find_user_by_username_query(
    conn: &mut PgConnection,
    username: &str,
) -> Result<UserRow> {
    let row = sqlx::query_as::<_, UserRow>(include_str!("./user_by_username.sql"))
        .bind(username)
        .fetch_one(conn)
        .await?;

    Ok(row)
}
