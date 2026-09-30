use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, users::find_user_by_id::*};
use oxidauth_repository::users::select_user_by_id_query::SelectUserByIdQuery;

pub struct FindUserByIdUseCase<T>
where
    T: SelectUserByIdQuery,
{
    users: T,
}

impl<T> FindUserByIdUseCase<T>
where
    T: SelectUserByIdQuery,
{
    pub fn new(users: T) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<T> FindUserByIdServiceTrait for FindUserByIdUseCase<T>
where
    T: SelectUserByIdQuery,
{
    #[tracing::instrument(name = "FindUserByIdUseCase::find_user_by_id", skip(self))]
    async fn find_user_by_id(&self, req: &FindUserById) -> Result<User, BoxedError> {
        self.users
            .select_user_by_id(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::users::{UserKind, UserStatus};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn user_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    fn stored_user() -> User {
        User {
            id: user_id(),
            kind: UserKind::Human,
            status: UserStatus::Enabled,
            username: "octocat".to_owned(),
            email: None,
            first_name: None,
            last_name: None,
            profile: json!(null),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Default)]
    struct Log {
        calls: Vec<Uuid>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn calls(log: &SharedLog) -> Vec<Uuid> {
        log.lock()
            .expect("log")
            .calls
            .clone()
    }

    struct MockUsers {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl SelectUserByIdQuery for MockUsers {
        async fn select_user_by_id(&self, req: &FindUserById) -> Result<User, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .calls
                .push(req.user_id);

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(stored_user())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> FindUserByIdUseCase<MockUsers> {
        FindUserByIdUseCase::new(MockUsers {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate (`self.users.select_user_by_id(req).await`) — passthrough + error
    // propagation is the whole contract.
    #[tokio::test]
    async fn request_id_reaches_the_query_and_the_row_is_returned_verbatim() {
        let log = Arc::new(Mutex::new(Log::default()));

        let user = use_case(&log, false)
            .find_user_by_id(&FindUserById { user_id: user_id() })
            .await
            .expect("find succeeds");

        assert_eq!(calls(&log), vec![user_id()]);
        assert_eq!(user.id, user_id());
        assert_eq!(user.username, "octocat");
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .find_user_by_id(&FindUserById { user_id: user_id() })
            .await
            .expect_err("select failure propagates");

        assert_eq!(error.to_string(), "simulated select failure");
        assert_eq!(calls(&log), vec![user_id()]);
    }
}
