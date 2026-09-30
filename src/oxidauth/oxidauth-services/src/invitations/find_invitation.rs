use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{
        Invitation,
        find_invitation::{FindInvitationParams, FindInvitationServiceTrait},
    },
};
use oxidauth_repository::invitations::select_invitation_by_id::SelectInvitationByIdQuery;

pub struct FindInvitationUseCase<T>
where
    T: SelectInvitationByIdQuery,
{
    select_invitation_by_id: T,
}

impl<T> FindInvitationUseCase<T>
where
    T: SelectInvitationByIdQuery,
{
    pub fn new(select_invitation_by_id: T) -> Self {
        Self {
            select_invitation_by_id,
        }
    }
}

#[async_trait]
impl<T> FindInvitationServiceTrait for FindInvitationUseCase<T>
where
    T: SelectInvitationByIdQuery,
{
    #[tracing::instrument(name = "FindInvitationUseCase::find_invitation", skip(self))]
    async fn find_invitation(
        &self,
        params: &FindInvitationParams,
    ) -> Result<Invitation, BoxedError> {
        self.select_invitation_by_id
            .select_invitation_by_id(params)
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
        uuid::uuid!("cccccccc-cccc-4ccc-8ccc-cccccccccccc")
    }

    #[derive(Default)]
    struct Log {
        finds: Vec<Uuid>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn finds(log: &SharedLog) -> Vec<Uuid> {
        log.lock()
            .expect("log")
            .finds
            .clone()
    }

    struct MockSelectById {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl SelectInvitationByIdQuery for MockSelectById {
        async fn select_invitation_by_id(
            &self,
            req: &FindInvitationParams,
        ) -> Result<Invitation, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .finds
                .push(req.invitation_id);

            if self.fail {
                return Err("RowNotFound".into());
            }

            Ok(Invitation {
                id: req.invitation_id,
                user_id: Uuid::new_v4(),
                expires_at: Utc::now(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> FindInvitationUseCase<MockSelectById> {
        FindInvitationUseCase::new(MockSelectById {
            log: log.clone(),
            fail,
        })
    }

    #[tokio::test]
    async fn existing_invitation_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));
        let params = FindInvitationParams {
            invitation_id: invitation_id(),
        };

        let found = use_case(&log, false)
            .find_invitation(&params)
            .await
            .expect("find succeeds");

        assert_eq!(found.id, invitation_id());
        assert_eq!(finds(&log), vec![invitation_id()]);
    }

    #[tokio::test]
    async fn missing_invitation_surfaces_the_repository_error_verbatim() {
        // No service-level not-found mapping: the repository RowNotFound
        // (select_invitation_by_id on a missing row) surfaces as-is.
        let log = Arc::new(Mutex::new(Log::default()));
        let params = FindInvitationParams {
            invitation_id: invitation_id(),
        };

        let error = use_case(&log, true)
            .find_invitation(&params)
            .await
            .expect_err("missing row must fail");

        assert_eq!(error.to_string(), "RowNotFound");
        assert_eq!(finds(&log), vec![invitation_id()]);
    }
}
