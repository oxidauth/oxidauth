use oxidauth_repository::user_role_grants::insert_user_role_grant::*;

use crate::prelude::*;

#[async_trait]
impl InsertUserRoleGrant for Database {
    async fn insert_user_role_grant(
        &self,
        params: &InsertUserRoleGrantParams,
    ) -> Result<UserRoleGrantRow, InsertUserRoleGrantError> {
        let result = sqlx::query_as::<_, super::UserRoleGrantRow>(include_str!(
            "./insert_user_role_grant.sql"
        ))
        .bind(params.user_id)
        .bind(params.role_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| InsertUserRoleGrantError {})?;

        Ok(result)
    }
}
