use oxidauth_repository::authorities::update_authority::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a UpdateAuthority> for Database {
    type Response = Authority;
    type Error = BoxedError;

    #[tracing::instrument(name = "update_authority_query", skip(self))]
    async fn call(
        &self,
        params: &'a UpdateAuthority,
    ) -> Result<Authority, BoxedError> {
        let result =
            sqlx::query_as::<_, PgAuthority>(include_str!("./update_authority.sql"))
                .bind(&params.id)
                .bind(&params.name)
                .bind(&params.client_key)
                .bind(params.status.as_ref().map(|s| s.to_string()))
                .bind(&params.strategy.to_string())
                .bind(serde_json::to_value(&params.settings)?)
                .bind(&params.params)
                .fetch_one(&self.pool)
                .await?;

        let authority = result.try_into()?;

        Ok(authority)
    }
}

#[cfg(test)]
mod tests {
    
    use sqlx::PgPool;

    

    #[ignore]
    #[sqlx::test]
    async fn it_should_update_an_authority_successfully(_pool: PgPool) {}
}
