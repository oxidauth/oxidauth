use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, users::delete_user_by_id::*};
use oxidauth_repository::users::delete_user_by_id_query::DeleteUserByIdQuery;

pub struct DeleteUserByIdUseCase<T>
where
    T: DeleteUserByIdQuery,
{
    users: T,
}

impl<T> DeleteUserByIdUseCase<T>
where
    T: DeleteUserByIdQuery,
{
    pub fn new(users: T) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<T> DeleteUserByIdServiceTrait for DeleteUserByIdUseCase<T>
where
    T: DeleteUserByIdQuery,
{
    #[tracing::instrument(name = "DeleteUserByIdUseCase::delete_user_by_id", skip(self))]
    async fn delete_user_by_id(&self, req: &DeleteUserById) -> Result<User, BoxedError> {
        self.users
            .delete_user_by_id(req)
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

    fn deleted_user() -> User {
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
        deletes: Vec<Uuid>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn deletes(log: &SharedLog) -> Vec<Uuid> {
        log.lock()
            .expect("log")
            .deletes
            .clone()
    }

    struct MockUsers {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl DeleteUserByIdQuery for MockUsers {
        async fn delete_user_by_id(&self, req: &DeleteUserById) -> Result<User, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .deletes
                .push(req.user_id);

            if self.fail {
                return Err("simulated delete failure".into());
            }

            Ok(deleted_user())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> DeleteUserByIdUseCase<MockUsers> {
        DeleteUserByIdUseCase::new(MockUsers {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate: id passed through, deleted row returned (the only
    // observable contract at this layer).
    #[tokio::test]
    async fn id_reaches_the_query_and_the_deleted_row_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));

        let user = use_case(&log, false)
            .delete_user_by_id(&DeleteUserById { user_id: user_id() })
            .await
            .expect("delete succeeds");

        assert_eq!(deletes(&log), vec![user_id()]);
        assert_eq!(user.id, user_id());
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        // FK-referenced users (user_role_grants etc.) fail at the database;
        // that error must reach callers unchanged
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .delete_user_by_id(&DeleteUserById { user_id: user_id() })
            .await
            .expect_err("delete failure propagates");

        assert_eq!(error.to_string(), "simulated delete failure");
        assert_eq!(deletes(&log), vec![user_id()]);
    }
}
