use crate::prelude::*;

impl Database {
    async fn find_permission_by_parts(&self, parts: PermissionParts) -> Result<PermissionRow> {
        let mut conn = self.pool.acquire().await?;

        find_permission_by_parts_query(&mut conn, parts).await
    }
}

pub async fn find_permission_by_parts_query(
    conn: &mut PgConnection,
    parts: PermissionParts,
) -> Result<PermissionRow> {
    let row = sqlx::query_as::<_, PermissionRow>(include_str!("./permission_by_parts.sql"))
        .bind(parts.realm)
        .bind(parts.resource)
        .bind(parts.action)
        .fetch_one(conn)
        .await?;

    Ok(row)
}
