use oxidauth_repository::authorities::delete_authority_by_id::*;

use crate::prelude::*;

#[async_trait]
impl DeleteAuthorityById for Database {
    async fn delete_authority_by_id(
        &self,
        authority_id: Uuid,
    ) -> Result<AuthorityRow, DeleteAuthorityByIdError> {
        let result =
            sqlx::query_as::<_, super::AuthorityRow>(include_str!("./delete_authority_by_id.sql"))
                .bind(authority_id)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| DeleteAuthorityByIdError {})?;

        Ok(result)
    }
}
