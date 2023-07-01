use oxidauth_repository::user_permission_grants::delete_user_permission_grant::*;

use crate::prelude::*;

#[async_trait]
impl DeleteUserPermissionGrant for Database {
    async fn delete_user_permission_grant(
        &self,
        params: &DeleteUserPermissionGrantParams,
    ) -> Result<UserPermissionGrantRow, DeleteUserPermissionGrantError> {
        let result = sqlx::query_as::<_, super::UserPermissionGrantRow>(include_str!(
            "./delete_user_permission_grant.sql"
        ))
        .bind(params.user_id)
        .bind(params.permission_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| DeleteUserPermissionGrantError {})?;

        Ok(result)
    }
}

// @GEORGE - delete where user OR permission match?
