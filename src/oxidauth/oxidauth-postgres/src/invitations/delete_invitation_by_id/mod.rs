use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, invitations::delete_invitation::DeleteInvitationParams};
use oxidauth_repository::invitations::delete_invitation_by_id::*;

use super::*;

#[async_trait]
impl DeleteInvitationByIdQuery for PgInvitationRepository {
    #[tracing::instrument(name = "delete_invitation_by_id_query", skip(self))]
    async fn delete_invitation_by_id(
        &self,
        params: &DeleteInvitationParams,
    ) -> Result<Invitation, BoxedError> {
        let result =
            sqlx::query_as::<_, PgInvitation>(include_str!("./delete_invitation_by_id_query.sql"))
                .bind(params.id)
                .fetch_one(&self.db.write_pool())
                .await?;

        let invitation = result.into();

        Ok(invitation)
    }
}

#[cfg(test)]
mod tests {

    use chrono::DateTime;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::{Database, test_fixtures::seed_user};

    fn repo(pool: &PgPool) -> PgInvitationRepository {
        PgInvitationRepository::new(Database::from_pool(pool.clone()))
    }

    async fn seed_invitation(pool: &PgPool) -> Uuid {
        let user = seed_user(pool).await;

        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO invitations (id, user_id, expires_at) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(user)
            .bind(DateTime::from_timestamp(1_893_456_000, 0).expect("valid ts"))
            .execute(pool)
            .await
            .expect("seed invitation");
        id
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_delete_and_return_the_removed_invitation(pool: PgPool) {
        let id = seed_invitation(&pool).await;

        let deleted = repo(&pool)
            .delete_invitation_by_id(&DeleteInvitationParams { id })
            .await
            .expect("delete should succeed");

        assert_eq!(deleted.id, id, "RETURNING gives the caller the removed row");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invitations")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 0);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_when_the_invitation_is_missing(pool: PgPool) {
        // deleting twice is not idempotent: the second pass sees no RETURNING row
        let id = seed_invitation(&pool).await;
        repo(&pool)
            .delete_invitation_by_id(&DeleteInvitationParams { id })
            .await
            .expect("first delete should succeed");

        let err = repo(&pool)
            .delete_invitation_by_id(&DeleteInvitationParams { id })
            .await
            .expect_err("second delete must surface RowNotFound");
        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .unwrap_or_else(|| panic!("expected a sqlx::Error, got: {err:?}"));
        assert!(
            matches!(sqlx_err, sqlx::Error::RowNotFound),
            "expected RowNotFound, got: {sqlx_err:?}"
        );
    }
}
