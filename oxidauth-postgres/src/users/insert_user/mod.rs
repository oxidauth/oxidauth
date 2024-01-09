use oxidauth_repository::users::insert_user::*;
use serde_json::Map;

use crate::Database;

use super::{TryFromUserRowError, UserRow};

impl<'a> Service<&'a InsertUserParams> for Database {
    type Response = User;

    type Error = InsertUserError;

    async fn call(
        &self,
        params: &'a InsertUserParams,
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
        .await
        .map_err(|err| InsertUserError {
            reason: "query failed".to_owned(),
            source: err.into(),
        })?;

        let user = row
            .try_into()
            .map_err(
                |err: TryFromUserRowError| InsertUserError {
                    reason: "converting user row to user failed".to_owned(),
                    source: Box::new(err),
                },
            )?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_be_able_to_insert_a_new_user(pool: PgPool) {
        let db = Database::new(pool)
            .expect("should be able to make a database from a pool");

        let username = "oxidauth_user";
        let insert_user: InsertUserParams = username.into();

        let result = db.call(&insert_user).await;

        match result {
            Ok(user) => assert_eq!(user.username, username),
            Err(_) => unreachable!(),
        }
    }

    #[ignore]
    #[sqlx::test]
    async fn it_should_fail_to_parse_an_invalid_user_status(pool: PgPool) {}

    #[ignore]
    #[sqlx::test]
    async fn it_should_fail_to_parse_an_invalid_user_kind(pool: PgPool) {}
}
