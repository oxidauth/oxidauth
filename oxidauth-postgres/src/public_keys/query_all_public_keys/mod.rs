use oxidauth_repository::public_keys::query_all_public_keys::*;

use crate::prelude::*;

#[async_trait]
impl QueryAllPublicKeys for Database {
    async fn query_all_public_keys(&self) -> Result<PublicKeyRow, QueryAllPublicKeysError> {
        let result =
            sqlx::query_as::<_, super::PublicKeyRow>(include_str!("./query_all_public_keys.sql"))
                .fetch_all(&self.pool)
                .await
                .map_err(|_| QueryAllPublicKeysError {})?
                .into_iter()
                .map(Into::into)
                .collect();

        Ok(result)
    }
}
