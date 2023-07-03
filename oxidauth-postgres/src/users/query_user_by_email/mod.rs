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

#[cfg(test)]
mod tests {
    use oxidauth_repository::users::insert_user::*;
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_query_a_user_by_email_successfully(pool: PgPool) {
        let db = Database { pool };

        let user_id = Uuid::new_v4();
        let user_email = "test@test.com".to_string();

        let insert_params = InsertUserParams {
            id: Some(user_id),
            kind: Some("Test".to_string()),
            status: Some("Test".to_string()),
            username: "Test".to_string(),
            email: Some(user_email),
            first_name: Some("TestFirst".to_string()),
            last_name: Some("TestLast".to_string()),
            profile: Some(serde_json::Value::default()),
        };

        db.insert_user(&insert_params)
            .await
            .expect("should be able to insert user");

        match db.query_user_by_email(user_email).await {
            Ok(user) => {
                assert_eq!(user_id, user.id);
                assert_eq!(insert_params.last_name, user.last_name);
                assert_eq!(insert_params.kind, user.kind);
                assert_eq!(insert_params.username, user.username);
            }
            Err(_) => unreachable!(),
        }
    }
}
