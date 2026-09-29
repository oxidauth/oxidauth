use crate::Database;

use oxidauth_kernel::{error::BoxedError, users::update_user::UpdateUser};
use oxidauth_repository::users::insert_user::*;

use super::UserRow;

#[async_trait]
impl<'a> Service<&'a UpdateUser> for Database {
    type Response = User;
    type Error = BoxedError;

    #[tracing::instrument(name = "update_user_query", skip(self))]
    async fn call(
        &self,
        params: &'a UpdateUser,
    ) -> Result<Self::Response, Self::Error> {
        let status: Option<&str> = params
            .status
            .as_ref()
            .map(|s| s.into());

        let row = sqlx::query_as::<_, UserRow>(include_str!(
            "./update_user.sql"
        ))
        .bind(params.id)
        .bind(&params.username)
        .bind(&params.email)
        .bind(&params.first_name)
        .bind(&params.last_name)
        .bind(status)
        .bind(&params.profile)
        .fetch_one(&self.write_pool())
        .await?;

        let user = row.try_into()?;

        Ok(user)
    }
}



