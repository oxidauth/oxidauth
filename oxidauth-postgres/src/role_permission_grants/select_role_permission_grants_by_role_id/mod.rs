use oxidauth_kernel::role_permission_grants::list_role_permission_grants_by_role_id::*;
use oxidauth_repository::role_permission_grants::select_role_permission_grants_by_role_id::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a ListRolePermissionGrantsByRoleId> for Database {
    type Response = Vec<RolePermissionGrantDetail>;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_role_role_grants_by_parent_id_query", skip(self))]
    async fn call(
        &self,
        params: &'a ListRolePermissionGrantsByRoleId,
    ) -> Result<Vec<RolePermissionGrantDetail>, BoxedError> {
        let result = sqlx::query_as::<_, PgRolePermissionGrantDetail>(include_str!(
            "./select_role_permission_grants_by_role_id.sql"
        ))
        .bind(&params.role_id)
        .fetch_all(&self.pool)
        .await?;

        let role_role_grant = result
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(role_role_grant)
    }
}
