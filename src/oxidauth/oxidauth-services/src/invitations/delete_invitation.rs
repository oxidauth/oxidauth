use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{
        Invitation,
        delete_invitation::{DeleteInvitationParams, DeleteInvitationServiceTrait},
    },
};
use oxidauth_repository::invitations::delete_invitation_by_id::DeleteInvitationByIdQuery;

pub struct DeleteInvitationUseCase<T>
where
    T: DeleteInvitationByIdQuery,
{
    delete_invitation_by_id: T,
}

impl<T> DeleteInvitationUseCase<T>
where
    T: DeleteInvitationByIdQuery,
{
    pub fn new(delete_invitation_by_id: T) -> Self {
        Self {
            delete_invitation_by_id,
        }
    }
}

#[async_trait]
impl<T> DeleteInvitationServiceTrait for DeleteInvitationUseCase<T>
where
    T: DeleteInvitationByIdQuery,
{
    #[tracing::instrument(name = "DeleteInvitationUseCase::delete_invitation", skip(self))]
    async fn delete_invitation(
        &self,
        params: &DeleteInvitationParams,
    ) -> Result<Invitation, BoxedError> {
        self.delete_invitation_by_id
            .delete_invitation_by_id(params)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use uuid::Uuid;

    use super::*;

    fn invitation_id() -> Uuid {
        uuid::uuid!("dddddddd-dddd-4ddd-8ddd-dddddddddddd")
    }

    #[derive(Default)]
    struct Log {
        deletes: Vec<Uuid>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn deletes(log: &SharedLog) -> Vec<Uuid> {
        log.lock()
            .expect("log")
            .deletes
            .clone()
    }

    struct MockDeleteById {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl DeleteInvitationByIdQuery for MockDeleteById {
        async fn delete_invitation_by_id(
            &self,
            req: &DeleteInvitationParams,
        ) -> Result<Invitation, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .deletes
                .push(req.id);

            if self.fail {
                // `DELETE FROM invitations WHERE id = $1 RETURNING *` with no
                // matching row (already accepted, already deleted, or never
                // existed) yields no row -> RowNotFound.
                return Err("RowNotFound".into());
            }

            Ok(Invitation {
                id: req.id,
                user_id: Uuid::new_v4(),
                expires_at: Utc::now(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> DeleteInvitationUseCase<MockDeleteById> {
        DeleteInvitationUseCase::new(MockDeleteById {
            log: log.clone(),
            fail,
        })
    }

    #[tokio::test]
    async fn delete_returns_the_removed_invitation() {
        let log = Arc::new(Mutex::new(Log::default()));
        let params = DeleteInvitationParams {
            id: invitation_id(),
        };

        let removed = use_case(&log, false)
            .delete_invitation(&params)
            .await
            .expect("delete succeeds");

        assert_eq!(removed.id, invitation_id());
        assert_eq!(deletes(&log), vec![invitation_id()]);
    }

    #[tokio::test]
    async fn missing_invitation_surfaces_the_repository_error_verbatim() {
        // No service-level not-found mapping and no idempotent no-op: a
        // second delete of the same id fails with RowNotFound.
        let log = Arc::new(Mutex::new(Log::default()));
        let params = DeleteInvitationParams {
            id: invitation_id(),
        };

        let error = use_case(&log, true)
            .delete_invitation(&params)
            .await
            .expect_err("missing row must fail");

        assert_eq!(error.to_string(), "RowNotFound");
        assert_eq!(deletes(&log), vec![invitation_id()]);
    }
}
