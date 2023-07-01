use oxidauth_repository::authorities::query_authority_by_id::*;

use crate::prelude::*;

#[async_trait]
impl QueryAuthorityById for Database {
    async fn query_authority_by_id(
        &self,
        authority_id: Uuid,
    ) -> Result<AuthorityRow, QueryAuthorityByIdError> {
        let result =
            sqlx::query_as::<_, super::AuthorityRow>(include_str!("./query_authority_by_id.sql"))
                .bind(authority_id)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| QueryAuthorityByIdError {})?;

        Ok(result)
    }
}
