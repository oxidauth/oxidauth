use crate::prelude::*;

impl Database {
    async fn all_permissions(&self) -> Result<Vec<PermissionRow>> {
        let mut conn = self.pool.acquire().await?;

        all_permissions_query(&mut conn).await
    }
}

pub async fn all_permissions_query(conn: &mut PgConnection) -> Result<Vec<PermissionRow>> {
    let rows = sqlx::query_as::<_, PermissionRow>(include_str!("./all_permissions.sql"))
        .fetch_all(conn)
        .await?;

    Ok(rows)
}
