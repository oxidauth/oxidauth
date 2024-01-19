use oxidauth_repository::public_keys::select_public_key_by_id::*;

use crate::prelude::*;

use super::PgPublicSanitizedKey;

#[async_trait]
impl<'a> Service<&'a FindPublicKeyById> for Database {
    type Response = PublicKey;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_public_key_by_id_query", skip(self))]
    async fn call(
        &self,
        public_key_id: &'a FindPublicKeyById,
    ) -> Result<Self::Response, Self::Error> {
        let public_key = sqlx::query_as::<_, PgPublicSanitizedKey>(
            include_str!("./select_public_key_by_id.sql"),
        )
        .bind(public_key_id.public_key_id)
        .fetch_one(&self.pool)
        .await?
        .try_into()?;

        Ok(public_key)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::public_keys::insert_public_key::*;
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_a_public_key_by_id_successfully(pool: PgPool) {}
}
