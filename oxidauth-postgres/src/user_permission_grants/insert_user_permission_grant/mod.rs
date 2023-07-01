use oxidauth_repository::user_permission_grants::insert_user_permission_grant::*;

use crate::prelude::*;

#[async_trait]
impl InsertUserPermissionGrant for Database {
    async fn insert_user_permission_grant(
        &self,
        params: &InsertUserPermissionGrantParams,
    ) -> Result<UserPermissionGrantRow, InsertUserPermissionGrantError> {
        let result = sqlx::query_as::<_, super::UserPermissionGrantRow>(include_str!(
            "./insert_user_permission_grant.sql"
        ))
        .bind(params.user_id)
        .bind(params.permission_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| InsertUserPermissionGrantError {})?;

        Ok(result)
    }
}
