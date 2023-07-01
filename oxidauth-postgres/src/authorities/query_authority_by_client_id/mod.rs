use oxidauth_repository::authorities::query_authority_by_client_id::*;

use crate::prelude::*;

#[async_trait]
impl QueryAuthorityByClientId for Database {
    async fn query_authority_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<AuthorityRow, QueryAuthorityByClientIdError> {
        let result = sqlx::query_as::<_, super::AuthorityRow>(include_str!(
            "./query_authority_by_client_id.sql"
        ))
        .bind(client_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| QueryAuthorityByClientIdError {})?;

        Ok(result)
    }
}
