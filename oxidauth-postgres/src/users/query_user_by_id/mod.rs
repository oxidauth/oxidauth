use oxidauth_repository::users::query_user_by_id::*;

use crate::prelude::*;

#[async_trait]
impl QueryUserById for Database {
    async fn query_user_by_id(&self, id: Uuid) -> Result<UserRow, QueryUserByIdError> {
        let result = sqlx::query_as::<_, super::UserRow>(include_str!("./query_user_by_id.sql"))
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map(Into::into)
            .map_err(|_| QueryUserByIdError {})?;

        Ok(result)
    }
}
