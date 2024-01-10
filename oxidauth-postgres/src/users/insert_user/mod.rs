use serde_json::Map;

use crate::Database;

use oxidauth_kernel::{users::{create_user::CreateUser, UserKind, UserStatus}, error::BoxedError};
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
        .fetch_one(&self.pool)
        .await?;

        let user = row.try_into()?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_be_able_to_insert_a_new_user(pool: PgPool) {}

    #[ignore]
    #[sqlx::test]
    async fn it_should_fail_to_parse_an_invalid_user_status(pool: PgPool) {}

    #[ignore]
    #[sqlx::test]
    async fn it_should_fail_to_parse_an_invalid_user_kind(pool: PgPool) {}
}
