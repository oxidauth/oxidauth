use oxidauth_repository::users::query_user_by_email::*;

use crate::prelude::*;

#[async_trait]
impl QueryUserByEmail for Database {
    async fn query_user_by_email(&self, email: String) -> Result<UserRow, QueryUserByEmailError> {
        let result = sqlx::query_as::<_, super::UserRow>(include_str!("./query_user_by_email.sql"))
            .bind(email)
            .fetch_one(&self.pool)
            .await
            .map(Into::into)
            .map_err(|_| QueryUserByEmailError {})?;

        Ok(result)
    }
}
