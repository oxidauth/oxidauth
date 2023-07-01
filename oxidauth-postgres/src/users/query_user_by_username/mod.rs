use oxidauth_repository::users::query_user_by_username::*;

use crate::prelude::*;

#[async_trait]
impl QueryUserByUsername for Database {
    async fn query_user_by_username(
        &self,
        username: String,
    ) -> Result<UserRow, QueryUserByUsernameError> {
        let result =
            sqlx::query_as::<_, super::UserRow>(include_str!("./query_user_by_username.sql"))
                .bind(username)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| QueryUserByUsernameError {})?;

        Ok(result)
    }
}
