use async_trait::async_trait;
use oxidauth_kernel::error::BoxedError;
use oxidauth_repository::users::select_user_by_username_query::*;

use super::*;

#[async_trait]
impl SelectUserByUsernameQuery for PgUserRepository {
    #[tracing::instrument(name = "select_user_by_username_query", skip(self))]
    async fn select_user_by_username(
        &self,
        username: &Username,
    ) -> Result<Option<User>, BoxedError> {
        let result =
            sqlx::query_as::<_, UserRow>(include_str!("./select_user_by_username_query.sql"))
                .bind(&username.0)
                .fetch_optional(&self.db.read_pool())
                .await?;

        let user = result
            .map(TryInto::try_into)
            .transpose()?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_kernel::users::{UserKind, UserStatus, create_user::CreateUser};
    use oxidauth_repository::users::insert_user::*;
    use serde_json::json;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;

    fn repo(pool: &PgPool) -> PgUserRepository {
        PgUserRepository::new(Database::from_pool(pool.clone()))
    }

    async fn seed_user(pool: &PgPool, username: &str) -> Uuid {
        let params = CreateUser {
            id: None,
            kind: Some(UserKind::Human),
            status: Some(UserStatus::Enabled),
            username: username.to_owned(),
            email: Some(format!("{username}@example.com")),
            first_name: None,
            last_name: None,
            profile: Some(json!({})),
        };

        repo(pool)
            .insert_user(&params)
            .await
            .expect("seed user should succeed")
            .id
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_find_user_by_username(pool: PgPool) {
        let id = seed_user(&pool, "alice").await;

        let user = repo(&pool)
            .select_user_by_username(&Username("alice".to_owned()))
            .await
            .expect("query should succeed")
            .expect("user should be found");

        assert_eq!(user.id, id);
        assert_eq!(user.username, "alice");
        assert_eq!(user.email.as_deref(), Some("alice@example.com"));
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_none_for_missing_username(pool: PgPool) {
        let result = repo(&pool)
            .select_user_by_username(&Username("nobody".to_owned()))
            .await
            .expect("query should succeed");

        assert!(result.is_none(), "missing row must resolve to `None`");
    }
}
