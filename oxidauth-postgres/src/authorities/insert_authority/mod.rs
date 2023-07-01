use oxidauth_repository::authorities::insert_authority::*;

use crate::prelude::*;

#[async_trait]
impl InsertAuthority for Database {
    async fn insert_authority(
        &self,
        params: &InsertAuthorityParams,
    ) -> Result<AuthorityRow, InsertAuthorityError> {
        let result =
            sqlx::query_as::<_, super::AuthorityRow>(include_str!("./insert_authority.sql"))
                .bind(&params.name)
                .bind(&params.client_key)
                .bind(&params.status)
                .bind(&params.strategy)
                .bind(&params.settings)
                .bind(&params.params)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| InsertAuthorityError {})?;

        Ok(result)
    }
}
