use oxidauth_repository::role_role_grants::delete_role_role_grant::*;

use crate::prelude::*;

#[async_trait]
impl DeleteRoleRoleGrant for Database {
    async fn delete_role_role_grant(
        &self,
        params: &DeleteRoleRoleGrantParams,
    ) -> Result<RoleRoleGrantRow, DeleteRoleRoleGrantError> {
        let result = sqlx::query_as::<_, super::RoleRoleGrantRow>(include_str!(
            "./delete_role_role_grant.sql"
        ))
        .bind(params.parent_id)
        .bind(params.child_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| DeleteRoleRoleGrantError {})?;

        Ok(result)
    }
}
