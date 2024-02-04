use oxidauth_repository::authorities::select_all_authorities::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a ListAllAuthorities> for Database {
    type Response = Vec<Authority>;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_all_authorities_query", skip(self))]
    async fn call(
        &self,
        params: &'a ListAllAuthorities,
    ) -> Result<Vec<Authority>, BoxedError> {
        let result =
            sqlx::query_as::<_, PgAuthority>(include_str!("./select_all_authorities.sql"))
                .fetch_all(&self.pool)
                .await?;

        let authorities = result
                .into_iter()
                .map(|a| a.try_into())
                .collect::<Result<Vec<Authority>, BoxedError>>()?;

        Ok(authorities)
    }
}

#[cfg(test)]
mod tests {
    
    use sqlx::PgPool;

    

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_all_authorities_successfully(_pool: PgPool) {}
}
