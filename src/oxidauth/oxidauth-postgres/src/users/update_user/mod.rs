use oxidauth_repository::users::update_user::*;

use super::*;

#[async_trait]
impl UpdateUserQuery for PgUserRepository {
    #[tracing::instrument(name = "update_user_query", skip(self))]
    async fn update_user(&self, params: &UpdateUser) -> Result<User, BoxedError> {
        let status: Option<&str> = params
            .status
            .as_ref()
            .map(|s| s.into());

        let row = sqlx::query_as::<_, UserRow>(include_str!("./update_user.sql"))
            .bind(params.id)
            .bind(&params.username)
            .bind(&params.email)
            .bind(&params.first_name)
            .bind(&params.last_name)
            .bind(status)
            .bind(&params.profile)
            .fetch_one(&self.db.write_pool())
            .await?;

        let user = row.try_into()?;

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
    use crate::Database;

    fn repo(pool: &PgPool) -> PgUserRepository {
        PgUserRepository::new(Database::from_pool(pool.clone()))
    }

    async fn seed_user(pool: &PgPool) -> User {
        let params = CreateUser {
            id: None,
            kind: Some(UserKind::Human),
            status: Some(UserStatus::Enabled),
            username: "update_me".to_owned(),
            email: Some("old@example.com".to_owned()),
            first_name: Some("Old".to_owned()),
            last_name: Some("Name".to_owned()),
            profile: Some(json!({ "v": 1 })),
        };

        repo(pool)
            .insert_user(&params)
            .await
            .expect("seed user")
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_update_fields_and_map_status_string(pool: PgPool) {
        let seeded = seed_user(&pool).await;

        let params = UpdateUser {
            id: seeded.id,
            username: Some("updated".to_owned()),
            email: None,
            first_name: Some("New".to_owned()),
            last_name: Some("Name".to_owned()),
            status: Some(UserStatus::Disabled),
            profile: Some(json!({ "v": 2 })),
        };

        let updated = repo(&pool)
            .update_user(&params)
            .await
            .expect("update should succeed");

        assert_eq!(updated.id, seeded.id);
        assert_eq!(updated.username, "updated");
        assert_eq!(updated.email, None, "`None` must overwrite the column");
        assert_eq!(updated.first_name.as_deref(), Some("New"));
        assert_eq!(updated.last_name.as_deref(), Some("Name"));
        assert_eq!(updated.profile, json!({ "v": 2 }));

        let status: &str = (&updated.status).into();
        assert_eq!(status, "disabled");
        assert_eq!(updated.kind, UserKind::Human, "kind is untouched");

        // pin the status mapping as actually stored
        let stored_status: String = sqlx::query_scalar("SELECT status FROM users WHERE id = $1")
            .bind(seeded.id)
            .fetch_one(&pool)
            .await
            .expect("row should exist");
        assert_eq!(stored_status, "disabled");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_null_username_overwrite(pool: PgPool) {
        // BUG(pinned): update_user.sql overwrites every listed column unconditionally —
        // `username: None` writes NULL and violates the NOT NULL constraint instead of
        // keeping the existing value, so "partial" updates are impossible for NOT NULL
        // columns. Pinning current SQL-level overwrite semantics.
        let seeded = seed_user(&pool).await;

        let params = UpdateUser {
            id: seeded.id,
            username: None,
            email: Some("still@example.com".to_owned()),
            first_name: None,
            last_name: None,
            status: None,
            profile: None,
        };

        let err = repo(&pool)
            .update_user(&params)
            .await
            .expect_err("NULL username must violate the NOT NULL constraint");

        assert!(
            format!("{err:?}").contains("not-null constraint"),
            "expected NOT NULL violation, got: {err:?}"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_on_missing_user(pool: PgPool) {
        let params = UpdateUser {
            id: Uuid::new_v4(),
            username: Some("nobody".to_owned()),
            email: None,
            first_name: None,
            last_name: None,
            status: None,
            profile: None,
        };

        let err = repo(&pool)
            .update_user(&params)
            .await
            .expect_err("missing row must surface as an error");

        assert!(
            format!("{err:?}").contains("RowNotFound"),
            "expected RowNotFound, got: {err:?}"
        );
    }
}
