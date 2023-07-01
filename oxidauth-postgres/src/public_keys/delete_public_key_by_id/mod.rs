use oxidauth_repository::public_keys::delete_public_key_by_id::*;

use crate::prelude::*;

#[async_trait]
impl DeletePublicKeyById for Database {
    async fn delete_public_key_by_id(
        &self,
        public_key_id: Uuid,
    ) -> Result<PublicKeyRow, DeletePublicKeyByIdError> {
        let result =
            sqlx::query_as::<_, super::PublicKeyRow>(include_str!("./delete_public_key_by_id.sql"))
                .bind(public_key_id)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| DeletePublicKeyByIdError {})?;

        Ok(result)
    }
}
