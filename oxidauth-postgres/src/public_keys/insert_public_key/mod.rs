use oxidauth_repository::public_keys::insert_public_key::*;

use crate::prelude::*;

#[async_trait]
impl InsertPublicKey for Database {
    async fn insert_public_key(
        &self,
        params: &InsertPublicKeyParams,
    ) -> Result<PublicKeyRow, InsertPublicKeyError> {
        let result =
            sqlx::query_as::<_, super::PublicKeyRow>(include_str!("./insert_public_key.sql"))
                .bind(&params.private_key)
                .bind(&params.public_key)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| InsertPublicKeyError {})?;

        Ok(result)
    }
}
