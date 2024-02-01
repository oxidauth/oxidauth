use oxidauth_repository::authorities::insert_authority::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a CreateAuthority> for Database {
    type Response = Authority;
    type Error = BoxedError;

    #[tracing::instrument(name = "insert_authority_query", skip(self))]
    async fn call(
        &self,
        params: &'a CreateAuthority,
    ) -> Result<Authority, BoxedError> {
        let result = sqlx::query_as::<_, PgAuthority>(include_str!(
            "./insert_authority.sql"
        ))
        .bind(None::<Uuid>)
        .bind(&params.name)
        .bind(params.client_key)
        .bind(
            params
                .status
                .as_ref()
                .map(|s| s.to_string()),
        )
        .bind(&params.strategy.to_string())
        .bind(serde_json::to_value(
            &params.settings,
        )?)
        .bind(
            &params
                .params
                .clone()
                .inner_value(),
        )
        .fetch_one(&self.pool)
        .await?;

        let authority = result.try_into()?;

        Ok(authority)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_insert_an_authority_successfully(pool: PgPool) {}
}
