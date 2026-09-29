use oxidauth_repository::authorities::select_authority_by_strategy::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a FindAuthorityByStrategy> for Database {
    type Response = Option<Authority>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_authority_by_strategy_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a FindAuthorityByStrategy,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, PgAuthority>(include_str!(
            "./select_authority_by_strategy.sql"
        ))
        .bind(params.strategy.to_string())
        .fetch_optional(&self.pool)
        .await?;

        let authority = result
            .map(TryInto::try_into)
            .transpose()?;

        Ok(authority)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_an_authority_by_strategy_successfully(
        _pool: PgPool,
    ) {
    }
}
