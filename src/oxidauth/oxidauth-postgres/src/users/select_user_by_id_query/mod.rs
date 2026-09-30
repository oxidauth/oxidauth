use async_trait::async_trait;
use oxidauth_kernel::error::BoxedError;
use oxidauth_repository::users::select_user_by_id_query::*;
use sqlx::PgConnection;

use super::*;

#[async_trait]
impl SelectUserByIdQuery for PgUserRepository {
    #[tracing::instrument(name = "select_user_by_id_query", skip(self))]
    async fn select_user_by_id(&self, user_id: &FindUserById) -> Result<User, BoxedError> {
        let pool = self.db.read_pool();
        let mut conn = pool.acquire().await?;

        let result = select_user_by_id_query(&mut conn, user_id.user_id).await?;

        let user = result.try_into()?;

        Ok(user)
    }
}

pub async fn select_user_by_id_query(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<UserRow, BoxedError> {
    let result = sqlx::query_as::<_, UserRow>(include_str!("./select_user_by_id_query.sql"))
        .bind(user_id)
        .fetch_one(conn)
        .await?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use oxidauth_kernel::users::create_user::CreateUser;
    use oxidauth_repository::users::insert_user::*;
    use serde_json::json;
    use sqlx::PgPool;

    use super::*;

    fn repo(pool: &PgPool) -> PgUserRepository {
        PgUserRepository::new(Database::from_pool(pool.clone()))
    }

    async fn seed_user(pool: &PgPool, username: &str) -> User {
        let params = CreateUser {
            id: None,
            kind: None,
            status: None,
            username: username.to_owned(),
            email: Some(format!("{username}@example.com")),
            first_name: Some(username.to_owned()),
            last_name: None,
            profile: Some(json!({ "seeded": true })),
        };

        repo(pool)
            .insert_user(&params)
            .await
            .expect("seed user")
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_find_user_by_id(pool: PgPool) {
        let seeded = seed_user(&pool, "by_id_user").await;

        let params = FindUserById { user_id: seeded.id };

        let found = repo(&pool)
            .select_user_by_id(&params)
            .await
            .expect("query should succeed");

        assert_eq!(found.id, seeded.id);
        assert_eq!(found.username, "by_id_user");
        assert_eq!(found.email.as_deref(), Some("by_id_user@example.com"));
        assert_eq!(found.first_name.as_deref(), Some("by_id_user"));
        assert_eq!(found.profile, json!({ "seeded": true }));
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_on_missing_row(pool: PgPool) {
        let params = FindUserById {
            user_id: Uuid::new_v4(),
        };

        let err = repo(&pool)
            .select_user_by_id(&params)
            .await
            .expect_err("missing row must surface as an error");

        assert!(
            format!("{err:?}").contains("RowNotFound"),
            "expected RowNotFound, got: {err:?}"
        );
    }
}
