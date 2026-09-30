use async_trait::async_trait;
use oxidauth_kernel::error::BoxedError;
use oxidauth_repository::users::select_all_users_query::*;

use super::*;

#[async_trait]
impl SelectAllUsersQuery for PgUserRepository {
    #[tracing::instrument(name = "select_all_users_query", skip(self))]
    async fn select_all_users(&self, params: &ListAllUsers) -> Result<Vec<User>, BoxedError> {
        let result = sqlx::query_as::<_, UserRow>(include_str!("./select_all_users_query.sql"))
            .fetch_all(&self.db.read_pool())
            .await?;

        let users = result
            .into_iter()
            .map(|u| u.try_into())
            .collect::<Result<Vec<User>, TryFromUserRowError>>()?;

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    fn repo(pool: &PgPool) -> PgUserRepository {
        PgUserRepository::new(Database::from_pool(pool.clone()))
    }

    async fn seed_user(pool: &PgPool, username: &str, kind: &str, status: &str) {
        sqlx::query("INSERT INTO users (kind, status, username) VALUES ($1, $2, $3)")
            .bind(kind)
            .bind(status)
            .bind(username)
            .execute(pool)
            .await
            .expect("seed user");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_every_user_with_mapped_status(pool: PgPool) {
        seed_user(&pool, "all_enabled", "human", "enabled").await;
        seed_user(&pool, "all_invited", "human", "invited").await;
        seed_user(&pool, "all_disabled", "api", "disabled").await;
        // Guard (OXA-000019 precedent): force the tie — seed rows tie on created_at
        // only if NOW() never ticks between INSERTs, and while updated_at equals
        // created_at an updated_at-first orderer is indistinguishable from the
        // correct one. Pin every row's created_at to the newest one, then drop the
        // max-id row's updated_at below that tie: `ORDER BY updated_at ASC, id ASC`
        // must then hoist the max-id row to the front — observably wrong — while
        // the correct orderer never reads updated_at and keeps it last.
        sqlx::query(
            "UPDATE users SET created_at = (SELECT max(created_at) FROM users), \
             updated_at = CASE WHEN id = (SELECT id FROM users ORDER BY id DESC LIMIT 1) \
             THEN (SELECT max(created_at) FROM users) - INTERVAL '1 hour' \
             ELSE (SELECT max(created_at) FROM users) END",
        )
        .execute(&pool)
        .await
        .expect("diverge one row's updated_at");

        let repo = repo(&pool);
        let users = repo
            .select_all_users(&ListAllUsers)
            .await
            .expect("query should succeed");

        assert_eq!(users.len(), 3, "all seeded users must be returned");

        assert!(
            users
                .windows(2)
                .all(|w| (w[0].created_at, w[0].id) <= (w[1].created_at, w[1].id)),
            "rows must be returned in (created_at, id) order"
        );

        let again = repo
            .select_all_users(&ListAllUsers)
            .await
            .expect("repeated query should succeed");
        assert_eq!(
            users
                .iter()
                .map(|u| u.id)
                .collect::<Vec<_>>(),
            again
                .iter()
                .map(|u| u.id)
                .collect::<Vec<_>>(),
            "repeated calls must return identical id sequences"
        );

        let mut actual: Vec<(String, String, String)> = users
            .into_iter()
            .map(|u| {
                let kind: &str = (&u.kind).into();
                let status: &str = (&u.status).into();
                (u.username, kind.to_owned(), status.to_owned())
            })
            .collect();
        actual.sort();

        assert_eq!(
            actual,
            vec![
                (
                    "all_disabled".to_owned(),
                    "api".to_owned(),
                    "disabled".to_owned()
                ),
                (
                    "all_enabled".to_owned(),
                    "human".to_owned(),
                    "enabled".to_owned()
                ),
                (
                    "all_invited".to_owned(),
                    "human".to_owned(),
                    "invited".to_owned()
                ),
            ]
        );
    }
}
