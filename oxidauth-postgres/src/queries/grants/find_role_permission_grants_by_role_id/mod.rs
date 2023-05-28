use crate::{prelude::*, rows::grants::RolePermissionGrantRow};

impl Database {
    async fn find_role_permission_grants_by_role_id(
        &self,
        role_id: Uuid,
    ) -> Result<Vec<RolePermissionGrantRow>> {
        let mut conn = self.pool.acquire().await?;

        find_role_permission_grants_by_role_id_query(&mut conn, role_id).await
    }
}

pub async fn find_role_permission_grants_by_role_id_query(
    conn: &mut PgConnection,
    role_id: Uuid,
) -> Result<Vec<RolePermissionGrantRow>> {
    let row = sqlx::query_as::<_, RolePermissionGrantRow>(include_str!(
        "./find_role_permission_grants_by_role_id.sql"
    ))
    .bind(role_id)
    .fetch_all(conn)
    .await?;

    Ok(row)
}
