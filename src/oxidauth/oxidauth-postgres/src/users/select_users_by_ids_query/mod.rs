use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, users::find_users_by_ids::UsersByIds};
use oxidauth_repository::users::select_users_by_ids_query::*;
use sqlx::PgConnection;

use super::*;

#[async_trait]
impl SelectUsersByIdsQuery for PgUserRepository {
    #[tracing::instrument(name = "select_users_by_ids_query", skip(self))]
    async fn select_users_by_ids(&self, params: &FindUsersByIds) -> Result<UsersByIds, BoxedError> {
        let pool = self.db.read_pool();
        let mut conn = pool.acquire().await?;

        let user_rows = select_users_by_ids_query(&mut conn, &params.user_ids).await?;

        let user_ids_found: Vec<Uuid> = user_rows
            .clone()
            .into_iter()
            .map(|u| u.id)
            .collect();

        let mut user_ids_not_found = params.user_ids.clone();

        user_ids_not_found.retain(|id| !&user_ids_found.contains(id));

        let users = user_rows
            .into_iter()
            .map(|u| {
                u.try_into()
                    .map_err(|err: TryFromUserRowError| Box::new(err).into())
            })
            .collect::<Result<Vec<User>, BoxedError>>()?;

        Ok(UsersByIds {
            users,
            user_ids_not_found,
        })
    }
}

pub async fn select_users_by_ids_query(
    conn: &mut PgConnection,
    user_ids: &[Uuid],
) -> Result<Vec<UserRow>, BoxedError> {
    let result = sqlx::query_as::<_, UserRow>(include_str!("./select_users_by_ids_query.sql"))
        .bind(user_ids)
        .fetch_all(conn)
        .await?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;
    use crate::test_fixtures::create_user;

    fn repo(pool: &PgPool) -> PgUserRepository {
        PgUserRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_split_found_and_not_found_ids(pool: PgPool) {
        let first = create_user(&pool, "ids_first").await;
        let second = create_user(&pool, "ids_second").await;
        let missing = Uuid::new_v4();

        let params = FindUsersByIds {
            user_ids: vec![first, missing, second],
        };

        let result = repo(&pool)
            .select_users_by_ids(&params)
            .await
            .expect("query should succeed");

        let mut found: Vec<Uuid> = result
            .users
            .iter()
            .map(|u| u.id)
            .collect();
        found.sort();

        let mut expected = vec![first, second];
        expected.sort();

        assert_eq!(found, expected, "both existing users must be returned");
        assert_eq!(
            result.user_ids_not_found,
            vec![missing],
            "absent id must be reported as not found"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_empty_lists_for_empty_id_input(pool: PgPool) {
        create_user(&pool, "ids_unrelated").await;

        let params = FindUsersByIds {
            user_ids: Vec::new(),
        };

        let result = repo(&pool)
            .select_users_by_ids(&params)
            .await
            .expect("empty id list must not error");

        assert!(result.users.is_empty());
        assert!(
            result
                .user_ids_not_found
                .is_empty()
        );
    }
}
