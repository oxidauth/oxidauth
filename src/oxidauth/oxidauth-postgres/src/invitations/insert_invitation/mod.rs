use async_trait::async_trait;
use oxidauth_kernel::error::BoxedError;
use oxidauth_repository::invitations::insert_invitation::*;

use super::*;

#[async_trait]
impl InsertInvitationQuery for PgInvitationRepository {
    #[tracing::instrument(name = "insert_invitation_query", skip(self))]
    async fn insert_invitation(
        &self,
        params: &InsertInvitationParams,
    ) -> Result<Invitation, BoxedError> {
        let result = sqlx::query_as::<_, PgInvitation>(include_str!("./insert_invitation.sql"))
            .bind(params.id)
            .bind(params.user_id)
            .bind(params.expires_at)
            .fetch_one(&self.db.write_pool())
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
    use crate::{
        Database,
        test_fixtures::{assert_sql_state, seed_user},
    };

    fn repo(pool: &PgPool) -> PgInvitationRepository {
        PgInvitationRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_round_trip_id_and_expiry(pool: PgPool) {
        let user = seed_user(&pool).await;
        let id = Uuid::new_v4();
        // expiry drives acceptance/rejection upstream, so pin the whole-second value
        let expires_at = DateTime::from_timestamp(1_893_456_000, 0).expect("2030-01-01");

        let invitation = repo(&pool)
            .insert_invitation(&InsertInvitationParams {
                id: Some(id),
                user_id: user,
                expires_at,
            })
            .await
            .expect("insert should succeed");

        assert_eq!(invitation.id, id, "an explicit id is honoured");
        assert_eq!(invitation.user_id, user);
        assert_eq!(
            invitation.expires_at, expires_at,
            "expiry must survive the round-trip"
        );
        assert!(
            invitation
                .created_at
                .timestamp()
                > 0
        );
        assert!(invitation.updated_at >= invitation.created_at);

        let stored: DateTime<Utc> =
            sqlx::query_scalar("SELECT expires_at FROM invitations WHERE id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .expect("stored expiry");
        assert_eq!(stored, expires_at);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_generate_an_id_when_none_is_given(pool: PgPool) {
        let user = seed_user(&pool).await;
        let expires_at = DateTime::from_timestamp(1_893_456_000, 0).expect("valid ts");

        let invitation = repo(&pool)
            .insert_invitation(&InsertInvitationParams {
                id: None,
                user_id: user,
                expires_at,
            })
            .await
            .expect("insert should succeed");

        assert!(
            !invitation.id.is_nil(),
            "COALESCE($1, uuid_generate_v4()) fills the id"
        );
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invitations WHERE id = $1")
            .bind(invitation.id)
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 1);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_an_unknown_user_because_of_the_users_fk(pool: PgPool) {
        let err = repo(&pool)
            .insert_invitation(&InsertInvitationParams {
                id: None,
                user_id: Uuid::new_v4(),
                expires_at: DateTime::from_timestamp(1_893_456_000, 0).expect("valid ts"),
            })
            .await
            .expect_err("invitations_users_fk must reject an unknown user id");
        assert_sql_state(err, "23503");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invitations")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 0, "the rejected insert must leave no dangling row");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_restrict_user_deletion_while_an_invitation_is_pending(pool: PgPool) {
        let user = seed_user(&pool).await;
        let invitation = repo(&pool)
            .insert_invitation(&InsertInvitationParams {
                id: None,
                user_id: user,
                expires_at: DateTime::from_timestamp(1_893_456_000, 0).expect("valid ts"),
            })
            .await
            .expect("an invitation for a real user should succeed");

        // The FK is deliberately RESTRICT, not the siblings' CASCADE: deleting
        // the target user must not silently consume the outstanding offer.
        let err = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user)
            .execute(&pool)
            .await
            .err()
            .expect("the user delete must fail while the invitation is pending");
        assert_sql_state(err.into(), "23503");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invitations WHERE id = $1")
            .bind(invitation.id)
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(
            count, 1,
            "the invitation must survive the blocked user delete"
        );
    }
}
