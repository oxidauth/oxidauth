use oxidauth_kernel::role_role_grants::delete_role_role_grant::*;
use oxidauth_repository::role_role_grants::delete_role_role_grant::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a DeleteRoleRoleGrant> for Database {
    type Response = RoleRoleGrant;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_role_role_grant_query", skip(self))]
    async fn call(
        &self,
        params: &'a DeleteRoleRoleGrant,
    ) -> Result<RoleRoleGrant, BoxedError> {
        let result = sqlx::query_as::<_, PgRoleRoleGrant>(include_str!(
            "./delete_role_role_grant.sql"
        ))
        .bind(params.parent_id)
        .bind(params.child_id)
        .fetch_one(&self.pool)
        .await?;

        let role_role_grant = result.into();

        Ok(role_role_grant)
    }
}
