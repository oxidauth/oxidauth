use oxidauth_kernel::private_keys::find_most_recent_public_key::FindMostRecentPrivateKey;
use oxidauth_repository::private_keys::select_most_recent_private_key::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a FindMostRecentPrivateKey> for Database {
    type Response = PrivateKey;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_most_recent_private_key_query", skip(self))]
    async fn call(
        &self,
        _params: &'a FindMostRecentPrivateKey,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, PgPrivateKey>(
            include_str!("./select_most_recent_private_key.sql"),
        )
        .fetch_one(&self.pool)
        .await?;

        let private_key = result.into();

        Ok(private_key)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_most_recent_private_key_successfully(pool: PgPool) {}
}
