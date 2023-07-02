use oxidauth_repository::user_authorities::insert_user_authority::*;

use crate::prelude::*;

#[async_trait]
impl InsertUserAuthority for Database {
    async fn insert_user_authority(
        &self,
        params: &InsertUserAuthorityParams,
    ) -> Result<UserAuthorityRow, InsertUserAuthorityError> {
        let result = sqlx::query_as::<_, super::UserAuthorityRow>(include_str!(
            "./insert_user_authority.sql"
        ))
        .bind(&params.user_id)
        .bind(&params.authority_id)
        .bind(&params.user_identifier)
        .bind(&params.params)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| InsertUserAuthorityError {})?;

        Ok(result)
    }
}
