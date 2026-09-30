use oxidauth_kernel::users::{
    UserAlreadyExistsError,
    UserKind,
    UserStatus,
    Username,
    create_user::CreateUser,
};
use oxidauth_repository::users::insert_user::*;
use serde_json::Map;

use super::*;

#[async_trait]
impl InsertUserQuery for PgUserRepository {
    #[tracing::instrument(name = "insert_user_query", skip(self))]
    async fn insert_user(&self, params: &CreateUser) -> Result<User, BoxedError> {
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

        let row = sqlx::query_as::<_, UserRow>(include_str!("./insert_user.sql"))
            .bind(params.id)
            .bind(kind)
            .bind(status)
            .bind(&params.username)
            .bind(&params.email)
            .bind(&params.first_name)
            .bind(&params.last_name)
            .bind(profile)
            .fetch_one(&self.db.write_pool())
            .await
            .map_err(|err| -> BoxedError {
                // SQLSTATE 23505 (unique_violation) on `users_username_key`
                // specifically. `users.username` is UNIQUE and that index —
                // never a check-then-insert — is the arbiter, so its verdict is
                // translated here into the domain error, at the same boundary
                // where `RowNotFound` becomes
                // `UserAuthorityNotFoundError` (OXA-000050). Other 23505s stay
                // raw: `CreateUser.id` is client-supplyable on POST /users, so
                // a `users_pkey` collision is an id clash, not a taken
                // username, and must not claim otherwise.
                //
                // The sqlx error is deliberately NOT chained as the cause:
                // `into_error` renders `source()` into the response body, which
                // would put SQLSTATE 23505, the `users_username_key` constraint
                // name and Postgres's DETAIL (naming the username) straight back
                // on an unauthenticated endpoint. The raw text is logged here
                // instead, with the username `Display` omits.
                //
                // Follow-on, deliberately not this ticket: a 23505 raised
                // against `user_authorities` reaches the register flow through
                // the same non-transactional `?`-chain (an orphan `users` row
                // makes a later retry collide) and needs its own message —
                // "already registered with this authority" — and its own
                // decision.
                if let Some(db_err) = err
                    .as_database_error()
                    .filter(|db_err| {
                        db_err.code().as_deref() == Some("23505")
                            && db_err.constraint() == Some("users_username_key")
                    })
                {
                    tracing::warn!(
                        username = %params.username,
                        database_error = %db_err,
                        "insert_user rejected by the username unique constraint"
                    );

                    return UserAlreadyExistsError::username(&Username(params.username.clone()));
                }

                err.into()
            })?;

        let user = row.try_into()?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::Database;

    fn repo(pool: &PgPool) -> PgUserRepository {
        PgUserRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_insert_user_with_defaults(pool: PgPool) {
        let params = CreateUser {
            id: None,
            kind: None,
            status: None,
            username: "default_human".to_owned(),
            email: None,
            first_name: None,
            last_name: None,
            profile: None,
        };

        let user = repo(&pool)
            .insert_user(&params)
            .await
            .expect("insert with defaults should succeed");

        assert_ne!(user.id, Uuid::nil(), "id should be generated");
        assert_eq!(user.username, "default_human");
        assert_eq!(user.kind, UserKind::Human);
        let status: &str = (&user.status).into();
        assert_eq!(status, "enabled");
        assert_eq!(user.email, None);
        assert_eq!(user.profile, json!({}));
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_round_trip_explicit_kind_and_status_strings(pool: PgPool) {
        let id = Uuid::new_v4();
        let params = CreateUser {
            id: Some(id),
            kind: Some(UserKind::Api),
            status: Some(UserStatus::Invited),
            username: "api_client".to_owned(),
            email: Some("api@example.com".to_owned()),
            first_name: Some("Api".to_owned()),
            last_name: Some("Client".to_owned()),
            profile: Some(json!({ "tier": 2 })),
        };

        let user = repo(&pool)
            .insert_user(&params)
            .await
            .expect("insert with explicit values should succeed");

        assert_eq!(user.id, id, "explicit id must be honored");
        assert_eq!(user.kind, UserKind::Api);
        let status: &str = (&user.status).into();
        assert_eq!(status, "invited");
        assert_eq!(user.profile, json!({ "tier": 2 }));

        // pin the enum-to-string mapping as stored in the database
        let (kind, status): (String, String) =
            sqlx::query_as("SELECT kind, status FROM users WHERE id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .expect("row should exist");

        assert_eq!(kind, "api");
        assert_eq!(status, "invited");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_duplicate_username(pool: PgPool) {
        let params = CreateUser {
            id: None,
            kind: None,
            status: None,
            username: "dup_username".to_owned(),
            email: None,
            first_name: None,
            last_name: None,
            profile: None,
        };

        repo(&pool)
            .insert_user(&params)
            .await
            .expect("first insert should succeed");

        let err = repo(&pool)
            .insert_user(&params)
            .await
            .expect_err("duplicate username must fail");

        // the persistence boundary translated the driver verdict into the
        // domain error (OXA-000050): matchable by callers, sanitized on the wire
        let typed = err
            .downcast_ref::<UserAlreadyExistsError>()
            .expect("a duplicate username must surface UserAlreadyExistsError");
        assert_eq!(typed.username.0, "dup_username");
        assert_eq!(err.to_string(), "username is already taken");

        let debug = format!("{err:?}");
        for leaked in ["23505", "users_username_key", "duplicate key"] {
            assert!(
                !debug.contains(leaked),
                "the wire copy leaks {leaked}: {debug}"
            );
        }

        // the unique index is still what says no — the typed error is not a
        // false positive, and only the first row exists
        let raw_err = sqlx::query("INSERT INTO users (kind, status, username) VALUES ($1, $2, $3)")
            .bind("human")
            .bind("enabled")
            .bind("dup_username")
            .execute(&pool)
            .await
            .expect_err("the users_username_key index must still reject the duplicate");
        assert_eq!(
            raw_err
                .as_database_error()
                .and_then(|db_err| db_err.code())
                .as_deref(),
            Some("23505"),
            "expected unique_violation, got: {raw_err}"
        );

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE username = $1")
            .bind("dup_username")
            .fetch_one(&pool)
            .await
            .expect("count query should run");
        assert_eq!(count, 1);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_propagate_non_constraint_errors_raw(pool: PgPool) {
        // VARCHAR(64) overflow is SQLSTATE 22001, not a unique violation: only
        // 23505 is translated, everything else keeps the raw sqlx error
        let params = CreateUser {
            id: None,
            kind: None,
            status: None,
            username: "u".repeat(65),
            email: None,
            first_name: None,
            last_name: None,
            profile: None,
        };

        let err = repo(&pool)
            .insert_user(&params)
            .await
            .expect_err("an over-long username must fail");

        assert!(
            err.downcast_ref::<UserAlreadyExistsError>()
                .is_none(),
            "only a unique violation maps to the typed error: {err:?}"
        );
        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .expect("non-constraint failures must still propagate the sqlx error");
        assert_eq!(
            sqlx_err
                .as_database_error()
                .and_then(|db_err| db_err.code())
                .as_deref(),
            Some("22001"),
            "expected string_data_right_truncation, got: {sqlx_err}"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_propagate_primary_key_collision_raw(pool: PgPool) {
        // A 23505 against users_pkey — a client-supplied `id` re-POSTed with a
        // fresh username — is an id clash, not a taken username: only a
        // users_username_key violation maps to the typed error.
        let first = CreateUser {
            id: None,
            kind: None,
            status: None,
            username: "first_user".to_owned(),
            email: None,
            first_name: None,
            last_name: None,
            profile: None,
        };
        let inserted = repo(&pool)
            .insert_user(&first)
            .await
            .expect("first insert should succeed");

        let clash = CreateUser {
            id: Some(inserted.id),
            kind: None,
            status: None,
            username: "clash_user".to_owned(),
            email: None,
            first_name: None,
            last_name: None,
            profile: None,
        };

        let err = repo(&pool)
            .insert_user(&clash)
            .await
            .expect_err("reusing an existing id must fail");

        assert!(
            err.downcast_ref::<UserAlreadyExistsError>()
                .is_none(),
            "a users_pkey collision must NOT claim the username is taken: {err:?}"
        );
        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .expect("pkey collisions must propagate the raw sqlx error");
        assert_eq!(
            sqlx_err
                .as_database_error()
                .and_then(|db_err| db_err.code())
                .as_deref(),
            Some("23505"),
            "expected unique_violation on users_pkey, got: {sqlx_err}"
        );
    }
}
