use oxidauth_kernel::public_keys::{
    delete_public_key::DeletePublicKey, PublicKey,
};

use crate::{prelude::*, public_keys::PgPublicKey};

#[async_trait]
impl<'a> Service<&'a DeletePublicKey> for Database {
    type Response = PublicKey;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_public_key_query", skip(self))]
    async fn call(
        &self,
        params: &'a DeletePublicKey,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, PgPublicKey>(include_str!(
            "./delete_public_key.sql"
        ))
        .bind(params.public_key_id)
        .fetch_one(&self.pool)
        .await?;

        let public_key = result.try_into()?;

        Ok(public_key)
    }
}
