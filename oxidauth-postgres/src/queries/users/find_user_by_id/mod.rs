use uuid::Uuid;

use crate::prelude::*;

impl Database {
    async fn find_user_by_id(&self, user_id: Uuid) -> Result<()> {
        let mut conn = self.pool.acquire().await?;

        find_user_by_id_query(&mut conn, user_id).await
    }
}

pub async fn find_user_by_id_query(conn: &mut PgConnection, user_id: Uuid) -> Result<()> {
    sqlx::query_as::<_, UserRow>(include_str!("./user_by_id.sql"))
        .bind(user_id)
        .fetch_all(conn)
        .await?;

    todo!()
}
