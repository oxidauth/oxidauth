use oxidauth_repository::authorities::update_authority::*;

use crate::prelude::*;

#[async_trait]
impl UpdateAuthority for Database {
    async fn update_authority(
        &self,
        params: &UpdateAuthorityParams,
    ) -> Result<AuthorityRow, UpdateAuthorityError> {
        let result =
            sqlx::query_as::<_, super::AuthorityRow>(include_str!("./update_authority.sql"))
                .bind(&params.id)
                .bind(&params.name)
                .bind(&params.client_key)
                .bind(&params.status)
                .bind(&params.strategy)
                .bind(&params.settings)
                .bind(&params.params)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| UpdateAuthorityError {})?;

        Ok(result)
    }
}
