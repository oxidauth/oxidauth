use oxidauth_repository::role_permission_grants::insert_role_permission_grant::*;

use crate::prelude::*;

#[async_trait]
impl InsertRolePermissionGrant for Database {
    async fn insert_role_permission_grant(
        &self,
        params: &InsertRolePermissionGrantParams,
    ) -> Result<RolePermissionGrantRow, InsertRolePermissionGrantError> {
        let result = sqlx::query_as::<_, super::RolePermissionGrantRow>(include_str!(
            "./insert_role_permission_grant.sql"
        ))
        .bind(&params.role_id)
        .bind(&params.permission_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| InsertRolePermissionGrantError {})?;

        Ok(result)
    }
}
