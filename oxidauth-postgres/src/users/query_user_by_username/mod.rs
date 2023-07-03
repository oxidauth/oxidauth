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

#[cfg(test)]
mod tests {
    use oxidauth_repository::users::insert_user::*;
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_query_a_user_by_username_successfully(pool: PgPool) {
        let db = Database { pool };

        let user_id = Uuid::new_v4();
        let username = "Test".to_string();

        let insert_params = InsertUserParams {
            id: Some(user_id),
            kind: Some("Test".to_string()),
            status: Some("Test".to_string()),
            username: username,
            email: Some("test@test.com".to_string()),
            first_name: Some("TestFirst".to_string()),
            last_name: Some("TestLast".to_string()),
            profile: Some(serde_json::Value::default()),
        };

        db.insert_user(&insert_params)
            .await
            .expect("should be able to insert user");

        match db.query_user_by_username(username).await {
            Ok(user) => {
                assert_eq!(user_id, user.id);
                assert_eq!(insert_params.last_name, user.last_name);
                assert_eq!(insert_params.kind, user.kind);
                assert_eq!(insert_params.email, user.email);
            }
            Err(_) => unreachable!(),
        }
    }
}
