use oxidauth_repository::user_authorities::update_user_authority::*;

use crate::prelude::*;

#[async_trait]
impl UpdateUserAuthorityByUserId for Database {
    async fn update_user_authority_by_user_id(
        &self,
        params: &UpdateUserAuthorityByUserIdParams,
    ) -> Result<UserAuthorityRow, UpdateUserAuthorityError> {
        let result = sqlx::query_as::<_, super::UserAuthorityRow>(include_str!(
            "./update_user_authority.sql"
        ))
        .bind(&params.user_id)
        .bind(&params.authority_id)
        .bind(&params.user_identifier)
        .bind(&params.params)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| UpdateUserAuthorityError {})?;

        Ok(result)
    }
}
