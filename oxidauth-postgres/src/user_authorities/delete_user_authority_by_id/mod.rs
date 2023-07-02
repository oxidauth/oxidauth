use oxidauth_repository::user_authorities::delete_user_authority_by_id::*;

use crate::prelude::*;

#[async_trait]
impl DeleteUserAuthorityById for Database {
    async fn delete_user_authority_by_id(
        &self,
        user_authority_id: Uuid,
    ) -> Result<UserAuthorityRow, DeleteUserAuthorityByIdError> {
        let result = sqlx::query_as::<_, super::UserAuthorityRow>(include_str!(
            "./delete_user_authority_by_id.sql"
        ))
        .bind(user_authority_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| DeleteUserAuthorityByIdError {})?;

        Ok(result)
    }
}
