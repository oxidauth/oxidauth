use oxidauth_repository::user_authorities::query_user_authorities_by_user_id::*;

use crate::prelude::*;

use super::PgUserAuthority;

#[async_trait]
impl QueryUserAuthoritiesByUserId for Database {
    async fn query_user_authorities_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserAuthorityRow>, QueryUserAuthoritiesByUserIdError> {
        let result = sqlx::query_as::<_, PgUserAuthority>(include_str!(
            "./query_user_authorities_by_user_id.sql"
        ))
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| QueryUserAuthoritiesByUserIdError {})?
        .into_iter()
        .map(Into::into)
        .collect();

        Ok(result)
    }
}
