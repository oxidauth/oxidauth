use serde_json::Map;

use crate::Database;

use oxidauth_kernel::{
    error::BoxedError,
    users::{create_user::CreateUser, UserKind, UserStatus},
};
use oxidauth_repository::users::insert_user::*;

use super::UserRow;

#[async_trait]
impl<'a> Service<&'a CreateUser> for Database {
    type Response = User;
    type Error = BoxedError;

    #[tracing::instrument(name = "insert_user_query", skip(self))]
    async fn call(
        &self,
        params: &'a CreateUser,
    ) -> Result<Self::Response, Self::Error> {
        let kind: &str = match &params.kind {
            Some(kind) => kind.into(),
            None => (&UserKind::default()).into(),
        };
        let status: &str = match &params.status {
            Some(status) => status.into(),
            None => (&UserStatus::default()).into(),
        };

        let map = &Value::Object(Map::new());
        let profile = params
            .profile
            .as_ref()
            .unwrap_or(map);

        let row = sqlx::query_as::<_, UserRow>(include_str!(
            "./insert_user.sql"
        ))
        .bind(params.id)
        .bind(kind)
        .bind(status)
        .bind(&params.username)
        .bind(&params.email)
        .bind(&params.first_name)
        .bind(&params.last_name)
        .bind(profile)
        .fetch_one(&self.write_pool())
        .await?;

        let user = row.try_into()?;

        Ok(user)
    }
}



