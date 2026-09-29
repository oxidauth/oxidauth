use oxidauth_kernel::public_keys::PublicKey;
use oxidauth_repository::public_keys::insert_public_key::*;

use crate::prelude::*;

use super::PgPublicKey;

#[async_trait]
impl<'a> Service<&'a InsertPublicKeyParams> for Database {
    type Response = PublicKey;
    type Error = BoxedError;

    #[tracing::instrument(name = "insert_public_key_query", skip(self, params))]
    async fn call(
        &self,
        params: &'a InsertPublicKeyParams,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, PgPublicKey>(include_str!(
            "./insert_public_key.sql"
        ))
        .bind(params.id)
        .bind(&params.private_key)
        .bind(&params.public_key)
        .fetch_one(&self.write_pool())
        .await?;

        let public_key = result.try_into()?;

        Ok(public_key)
    }
}



