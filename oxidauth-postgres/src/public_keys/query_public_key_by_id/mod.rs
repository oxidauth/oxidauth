use oxidauth_repository::public_keys::query_public_key_by_id::*;

use crate::prelude::*;

#[async_trait]
impl QueryPublicKeyById for Database {
    async fn query_public_key_by_id(
        &self,
        public_key_id: Uuid,
    ) -> Result<PublicKeyRow, QueryPublicKeyByIdError> {
        let result =
            sqlx::query_as::<_, super::PublicKeyRow>(include_str!("./query_public_key_by_id.sql"))
                .bind(public_key_id)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| QueryPublicKeyByIdError {})?;

        Ok(result)
    }
}
