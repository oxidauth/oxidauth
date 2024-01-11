use oxidauth_repository::users::select_user_by_username_query::*;

use crate::prelude::*;

use super::UserRow;

#[async_trait]
impl<'a> Service<&'a Username> for Database {
    type Response = User;
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
        .fetch_one(&self.pool)
        .await?;

        let user = result.try_into()?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::users::insert_user::*;
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_a_user_by_username_successfully(pool: PgPool) {
        // let db = Database { pool };
        //
        // let user_id = Uuid::new_v4();
        //
        // let insert_params = InsertUserParams {
        //     id: Some(user_id),
        //     kind: Some("Test".to_string()),
        //     status: Some("Test".to_string()),
        //     username: "Test".to_string(),
        //     email: Some("test@test.com".to_string()),
        //     first_name: Some("TestFirst".to_string()),
        //     last_name: Some("TestLast".to_string()),
        //     profile: Some(serde_json::Value::default()),
        // };
        //
        // db.insert_user(&insert_params)
        //     .await
        //     .expect("should be able to insert user");
        //
        // match db.query_user_by_id(user_id).await {
        //     Ok(user) => {
        //         assert_eq!(user_id, user.id);
        //         assert_eq!(insert_params.last_name, user.last_name);
        //         assert_eq!(insert_params.kind, user.kind);
        //         assert_eq!(insert_params.email, user.email);
        //         assert_eq!(insert_params.username, user.username);
        //     }
        //     Err(_) => unreachable!(),
        // }
    }
}
