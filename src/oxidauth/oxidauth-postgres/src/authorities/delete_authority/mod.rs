use oxidauth_repository::authorities::delete_authority::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a DeleteAuthority> for Database {
    type Response = Authority;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_authority_query", skip(self))]
    async fn call(
        &self,
        params: &'a DeleteAuthority,
    ) -> Result<Authority, BoxedError> {
        let result = sqlx::query_as::<_, PgAuthority>(include_str!(
            "./delete_authority.sql"
        ))
        .bind(params.authority_id)
        .fetch_one(&self.write_pool())
        .await?;

        let authority = result.try_into()?;

        Ok(authority)
    }
}



