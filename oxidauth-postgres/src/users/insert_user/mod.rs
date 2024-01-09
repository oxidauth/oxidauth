use serde_json::Map;

use crate::Database;

use oxidauth_kernel::{users::create_user::CreateUser, error::BoxedError};
use oxidauth_repository::users::insert_user::*;

use super::{TryFromUserRowError, UserRow};

#[async_trait]
impl<'a> Service<&'a CreateUser> for Database {
    type Response = User;
    type Error = BoxedError;

    async fn call(
        &self,
        params: &'a CreateUser,
    ) -> Result<Self::Response, Self::Error> {
        let kind: Option<&str> = params
            .kind
            .as_ref()
            .map(|v| v.into());
        let status: Option<&str> = params
            .status
            .as_ref()
            .map(|v| v.into());

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
