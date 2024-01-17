use oxidauth_repository::role_permission_grants::delete_role_permission_grant::*;

use crate::prelude::*;

#[async_trait]
impl DeleteRolePermissionGrant for Database {
    async fn delete_role_permission_grant(
        &self,
        params: &DeleteRolePermissionGrantParams,
    ) -> Result<RolePermissionGrantRow, DeleteRolePermissionGrantError> {
        todo!()
        // let result = sqlx::query_as::<_, super::PgRolePermissionGrant>(include_str!(
        //     "./delete_role_permission_grant.sql"
        // ))
        // .bind(params.role_id)
        // .bind(params.permission_id)
        // .fetch_one(&self.pool)
        // .await
        // .map(Into::into)
        // .map_err(|_| DeleteRolePermissionGrantError {})?;
        //
        // Ok(result)
    }
}
