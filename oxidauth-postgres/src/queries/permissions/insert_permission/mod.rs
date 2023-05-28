use crate::prelude::*;

impl Database {
    async fn insert_permission(&self, permission: PermissionParts) -> Result<PermissionRow> {
        let mut conn = self.pool.acquire().await?;

        insert_permission_query(&mut conn, permission).await
    }
}

pub async fn insert_permission_query(
    conn: &mut PgConnection,
    permission: PermissionParts,
) -> Result<PermissionRow> {
    let rows = sqlx::query_as::<_, PermissionRow>(include_str!("./insert_permission.sql"))
        .bind(permission.realm)
        .bind(permission.resource)
        .bind(permission.action)
        .fetch_one(conn)
        .await?;

    Ok(rows)
}
