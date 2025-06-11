use oxidauth_kernel::authorities::find_authority_by_client_key::FindAuthorityByClientKey;
use oxidauth_repository::authorities::select_authority_by_client_key::*;
use tracing_log::log::info;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a FindAuthorityByClientKey> for Database {
    type Response = Option<Authority>;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_authority_by_client_key_query", skip(self))]
    async fn call(
        &self,
        params: &'a FindAuthorityByClientKey,
    ) -> Result<Self::Response, Self::Error> {
        let result =
            sqlx::query_as::<_, PgAuthority>(include_str!("./select_authority_by_client_key.sql"))
                .bind(params.client_key)
                .fetch_one(&self.pool)
                .await?;

        info!("Got FindAuthorityByClientKey result {:?}", result);

        let authority = result.try_into()?;

        info!("After try into {:?}", authority);

        Ok(Some(authority))
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_an_authority_by_client_key_successfully(_pool: PgPool) {}
}
