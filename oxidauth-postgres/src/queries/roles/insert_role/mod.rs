use crate::prelude::*;

impl Database {
    async fn insert_role(&self) -> Result<RoleRow> {
        let mut conn = self.pool.acquire().await?;

        insert_role_query(&mut conn).await
    }
}

pub async fn insert_role_query(conn: &mut PgConnection) -> Result<RoleRow> {
    let row = sqlx::query_as::<_, RoleRow>(include_str!("./insert_role.sql"))
        .fetch_one(conn)
        .await?;

    Ok(row)
}
