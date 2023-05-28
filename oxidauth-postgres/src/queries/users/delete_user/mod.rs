use uuid::Uuid;

use crate::prelude::*;

impl Database {
    pub async fn delete_user(&self, user_id: Uuid) -> Result<UserRow> {
        let mut conn = self.pool.acquire().await?;

        delete_user_query(&mut conn, user_id).await
    }
}

pub async fn delete_user_query(conn: &mut PgConnection, user_id: Uuid) -> Result<UserRow> {
    let row = sqlx::query_as::<_, UserRow>(include_str!("./delete_user.sql"))
        .bind(user_id)
        .fetch_one(conn)
        .await?;

    Ok(row)
}
