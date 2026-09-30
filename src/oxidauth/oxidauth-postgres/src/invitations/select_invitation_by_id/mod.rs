use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, invitations::find_invitation::FindInvitationParams};
use oxidauth_repository::invitations::select_invitation_by_id::*;

use super::*;

#[async_trait]
impl SelectInvitationByIdQuery for PgInvitationRepository {
    #[tracing::instrument(name = "select_invitation_by_id_query", skip(self))]
    async fn select_invitation_by_id(
        &self,
        params: &FindInvitationParams,
    ) -> Result<Invitation, BoxedError> {
        let result =
            sqlx::query_as::<_, PgInvitation>(include_str!("./select_invitation_by_id_query.sql"))
                .bind(params.invitation_id)
                .fetch_one(&self.db.read_pool())
                .await?;

        let invitation = result.into();

        Ok(invitation)
    }
}

#[cfg(test)]
mod tests {

    use chrono::{DateTime, Utc};
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::{Database, test_fixtures::seed_user};

    fn repo(pool: &PgPool) -> PgInvitationRepository {
        PgInvitationRepository::new(Database::from_pool(pool.clone()))
    }

    async fn seed_invitation(pool: &PgPool, expires_at: DateTime<Utc>) -> (Uuid, Uuid) {
        let user = seed_user(pool).await;

        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO invitations (id, user_id, expires_at) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(user)
            .bind(expires_at)
            .execute(pool)
            .await
            .expect("seed invitation");
        (id, user)
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_find_an_invitation_by_id(pool: PgPool) {
        let expires_at = DateTime::from_timestamp(1_893_456_000, 0).expect("valid ts");
        let (id, user) = seed_invitation(&pool, expires_at).await;

        let found = repo(&pool)
            .select_invitation_by_id(&FindInvitationParams { invitation_id: id })
            .await
            .expect("query should succeed");

        assert_eq!(found.id, id);
        assert_eq!(found.user_id, user);
        assert_eq!(found.expires_at, expires_at);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_when_the_invitation_is_missing(pool: PgPool) {
        let err = repo(&pool)
            .select_invitation_by_id(&FindInvitationParams {
                invitation_id: Uuid::new_v4(),
            })
            .await
            .expect_err("an unknown id must not return Ok");
        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .unwrap_or_else(|| panic!("expected a sqlx::Error, got: {err:?}"));
        assert!(
            matches!(sqlx_err, sqlx::Error::RowNotFound),
            "expected RowNotFound, got: {sqlx_err:?}"
        );
    }
}
