use crate::{
    prelude::*,
    rows::grants::{RolePermissionGrant, RolePermissionGrantRow},
};

impl Database {
    async fn insert_role_permission_grant(
        &self,
        params: RolePermissionGrant,
    ) -> Result<RolePermissionGrantRow> {
        let mut conn = self.pool.acquire().await?;

        insert_role_permission_grant_query(&mut conn, params).await
    }
}

pub async fn insert_role_permission_grant_query(
    conn: &mut PgConnection,
    params: RolePermissionGrant,
) -> Result<RolePermissionGrantRow> {
    let row = sqlx::query_as::<_, RolePermissionGrantRow>(include_str!(
        "./insert_role_permission_grant.sql"
    ))
    .bind(params.role_id)
    .bind(params.permission_id)
    .fetch_one(conn)
    .await?;

    Ok(row)
}
