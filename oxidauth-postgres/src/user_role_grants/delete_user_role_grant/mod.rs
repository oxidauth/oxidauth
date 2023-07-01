use oxidauth_repository::user_role_grants::delete_user_role_grant::*;

use crate::prelude::*;

#[async_trait]
impl DeleteUserRoleGrant for Database {
    async fn delete_user_role_grant(
        &self,
        params: &DeleteUserRoleGrantParams,
    ) -> Result<UserRoleGrantRow, DeleteUserRoleGrantError> {
        let result = sqlx::query_as::<_, super::UserRoleGrantRow>(include_str!(
            "./delete_user_role_grant.sql"
        ))
        .bind(params.user_id)
        .bind(params.role_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| DeleteUserRoleGrantError {})?;

        Ok(result)
    }
}
