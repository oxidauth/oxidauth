use crate::{
    prelude::*,
    rows::grants::{RolePermissionGrant, RolePermissionGrantRow},
};

impl Database {
    async fn delete_role_permission_grant(
        &self,
        params: RolePermissionGrant,
    ) -> Result<RolePermissionGrantRow> {
        let mut conn = self.pool.acquire().await?;

        delete_role_permission_grant_query(&mut conn, params).await
    }
}

pub async fn delete_role_permission_grant_query(
    conn: &mut PgConnection,
    params: RolePermissionGrant,
) -> Result<RolePermissionGrantRow> {
    let row = sqlx::query_as::<_, RolePermissionGrantRow>(include_str!(
        "./delete_role_permission_grant.sql"
    ))
    .bind(params.role_id)
    .bind(params.permission_id)
    .fetch_one(conn)
    .await?;

    Ok(row)
}
