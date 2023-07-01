use oxidauth_repository::role_role_grants::insert_role_role_grant::*;

use crate::prelude::*;

#[async_trait]
impl InsertRoleRoleGrant for Database {
    async fn insert_role_role_grant(
        &self,
        params: &InsertRoleRoleGrantParams,
    ) -> Result<RoleRoleGrantRow, InsertRoleRoleGrantError> {
        let result = sqlx::query_as::<_, super::RoleRoleGrantRow>(include_str!(
            "./insert_role_role_grant.sql"
        ))
        .bind(params.parent_id)
        .bind(params.child_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| InsertRoleRoleGrantError {})?;

        Ok(result)
    }
}
