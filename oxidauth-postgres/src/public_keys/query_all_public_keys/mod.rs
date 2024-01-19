use oxidauth_kernel::public_keys::PublicKey;
use oxidauth_repository::public_keys::query_all_public_keys::*;

use crate::prelude::*;

use super::PgPublicKey;

#[async_trait]
impl QueryAllPublicKeys for Database {
    async fn query_all_public_keys(
        &self,
    ) -> Result<Vec<PublicKey>, QueryAllPublicKeysError> {
        sqlx::query_as::<_, PgPublicKey>(include_str!(
            "./query_all_public_keys.sql"
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| QueryAllPublicKeysError {})?
        .into_iter()
        .map(|public_key| {
            public_key
                .try_into()
                .map_err(|_| QueryAllPublicKeysError {})
        })
        .collect::<Result<Vec<PublicKey>, QueryAllPublicKeysError>>()
    }
}
