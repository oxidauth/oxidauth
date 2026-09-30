use oxidauth_repository::refresh_tokens::delete_refresh_token_by_id::*;

use super::*;

#[async_trait]
impl DeleteRefreshTokenByIdQuery for PgRefreshTokenRepository {
    #[tracing::instrument(name = "delete_refresh_token_by_id_query", skip(self))]
    async fn delete_refresh_token_by_id(
        &self,
        params: &DeleteRefreshTokenById,
    ) -> Result<RefreshToken, BoxedError> {
        let result =
            sqlx::query_as::<_, PgRefreshToken>(include_str!("./delete_refresh_token_by_id.sql"))
                .bind(params.refresh_token_id)
                .fetch_one(&self.db.write_pool())
                .await?;

        let refresh_token = result.into();

        Ok(refresh_token)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::test_fixtures::{seed_authority, seed_refresh_token_now, seed_user};

    fn repo(pool: &PgPool) -> PgRefreshTokenRepository {
        PgRefreshTokenRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_delete_and_return_the_removed_token(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority(&pool, "refresh-del-id").await;
        let id = seed_refresh_token_now(&pool, user, authority).await;

        let deleted = repo(&pool)
            .delete_refresh_token_by_id(&DeleteRefreshTokenById {
                refresh_token_id: id,
            })
            .await
            .expect("delete should succeed");

        assert_eq!(deleted.id, id, "the removed row is returned via RETURNING");
        assert_eq!(deleted.user_id, user);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM refresh_tokens WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 0);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_when_the_token_is_already_gone(pool: PgPool) {
        // this call feeds the expired-token cleanup path: deleting twice is NOT a
        // silent no-op — DELETE ... RETURNING + fetch_one errors on the second pass
        let user = seed_user(&pool).await;
        let authority = seed_authority(&pool, "refresh-del-twice").await;
        let id = seed_refresh_token_now(&pool, user, authority).await;

        repo(&pool)
            .delete_refresh_token_by_id(&DeleteRefreshTokenById {
                refresh_token_id: id,
            })
            .await
            .expect("first delete should succeed");

        let err = repo(&pool)
            .delete_refresh_token_by_id(&DeleteRefreshTokenById {
                refresh_token_id: id,
            })
            .await
            .err()
            .unwrap_or_else(|| panic!("second delete must surface RowNotFound, got Ok"));
        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .unwrap_or_else(|| panic!("expected a sqlx::Error, got: {err:?}"));
        assert!(
            matches!(sqlx_err, sqlx::Error::RowNotFound),
            "expected RowNotFound, got: {sqlx_err:?}"
        );
    }
}
