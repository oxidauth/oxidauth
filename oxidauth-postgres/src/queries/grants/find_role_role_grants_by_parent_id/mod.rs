use crate::{prelude::*, rows::grants::RoleRoleGrantRow};

impl Database {
    async fn find_role_role_grants_by_parent_id(
        &self,
        role_id: Uuid,
    ) -> Result<Vec<RoleRoleGrantRow>> {
        let mut conn = self.pool.acquire().await?;

        find_role_role_grants_by_parent_id_query(&mut conn, role_id).await
    }
}

pub async fn find_role_role_grants_by_parent_id_query(
    conn: &mut PgConnection,
    role_id: Uuid,
) -> Result<Vec<RoleRoleGrantRow>> {
    let row = sqlx::query_as::<_, RoleRoleGrantRow>(include_str!(
        "./find_role_role_grants_by_parent_id.sql"
    ))
    .bind(role_id)
    .fetch_all(conn)
    .await?;

    Ok(row)
}
