use oxidauth_repository::public_keys::select_all_public_keys::*;

use crate::prelude::*;

use super::PgPublicSanitizedKey;

#[async_trait]
impl<'a> Service<&'a ListAllPublicKeys> for Database {
    type Response = Vec<PublicKey>;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_all_public_keys_query", skip(self))]
    async fn call(
        &self,
        public_key_id: &'a ListAllPublicKeys,
    ) -> Result<Self::Response, Self::Error> {
        let public_key = sqlx::query_as::<_, PgPublicSanitizedKey>(
            include_str!("./select_all_public_keys.sql"),
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|public_key| public_key.try_into())
        .collect::<Result<Vec<PublicKey>, BoxedError>>()?;

        Ok(public_key)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_all_public_keys_successfully(_pool: PgPool) {}
}
