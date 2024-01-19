use oxidauth_repository::authorities::select_authority_by_id::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a FindAuthorityById> for Database {
    type Response = Authority;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_authority_by_id_query", skip(self))]
    async fn call(
        &self,
        params: &'a FindAuthorityById,
    ) -> Result<Authority, BoxedError> {
        let result =
            sqlx::query_as::<_, PgAuthority>(include_str!("./query_authority_by_id.sql"))
                .bind(&params.authority_id)
                .fetch_one(&self.pool)
                .await?;

        let authority = result.try_into()?;

        Ok(authority)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::authorities::insert_authority::*;
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_an_authority_by_id_successfully(pool: PgPool) {}
}
