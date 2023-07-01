use oxidauth_repository::authorities::query_all_authorities::*;

use crate::prelude::*;

#[async_trait]
impl QueryAllAuthorities for Database {
    async fn query_all_authorities(&self) -> Result<AuthorityRow, QueryAllAuthoritiesError> {
        let result =
            sqlx::query_as::<_, super::AuthorityRow>(include_str!("./query_all_authorities.sql"))
                .fetch_all(&self.pool)
                .await
                .map_err(|_| QueryAllAuthoritiesError {})?
                .into_iter()
                .map(Into::into)
                .collect();

        Ok(result)
    }
}
