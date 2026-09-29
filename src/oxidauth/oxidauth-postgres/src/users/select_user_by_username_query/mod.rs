use oxidauth_repository::users::select_user_by_username_query::*;

use crate::prelude::*;

use super::UserRow;

#[async_trait]
impl<'a> Service<&'a Username> for Database {
    type Response = Option<User>;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_user_by_id_query", skip(self))]
    async fn call(
        &self,
        username: &'a Username,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, UserRow>(include_str!(
            "./select_user_by_username_query.sql"
        ))
        .bind(&username.0)
        .fetch_optional(&self.read_pool())
        .await?;

        let user = result
            .map(TryInto::try_into)
            .transpose()?;

        Ok(user)
    }
}



