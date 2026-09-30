use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, users::list_all_users::*};
use oxidauth_repository::users::select_all_users_query::SelectAllUsersQuery;

pub struct ListAllUsersUseCase<T>
where
    T: SelectAllUsersQuery,
{
    users: T,
}

impl<T> ListAllUsersUseCase<T>
where
    T: SelectAllUsersQuery,
{
    pub fn new(users: T) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<T> ListAllUsersServiceTrait for ListAllUsersUseCase<T>
where
    T: SelectAllUsersQuery,
{
    #[tracing::instrument(name = "ListAllUsersUseCase::list_all_users", skip(self))]
    async fn list_all_users(&self, req: &ListAllUsers) -> Result<Vec<User>, BoxedError> {
        self.users
            .select_all_users(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::users::{UserKind, UserStatus};
    use serde_json::json;

    use super::*;

    fn stored_users() -> Vec<User> {
        ["alice", "bob"]
            .iter()
            .map(|username| {
                User {
                    id: uuid::uuid!("11111111-1111-4111-8111-111111111111"),
                    kind: UserKind::Human,
                    status: UserStatus::Enabled,
                    username: (*username).to_owned(),
                    email: None,
                    first_name: None,
                    last_name: None,
                    profile: json!(null),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }
            })
            .collect()
    }

    #[derive(Default)]
    struct Log {
        calls: usize,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn calls(log: &SharedLog) -> usize {
        log.lock().expect("log").calls
    }

    struct MockUsers {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl SelectAllUsersQuery for MockUsers {
        async fn select_all_users(&self, params: &ListAllUsers) -> Result<Vec<User>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .calls += 1;

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(stored_users())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> ListAllUsersUseCase<MockUsers> {
        ListAllUsersUseCase::new(MockUsers {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate: one call, list returned untouched (no filtering,
    // sorting, or dedup happens at this layer).
    #[tokio::test]
    async fn the_stored_list_is_returned_in_query_order() {
        let log = Arc::new(Mutex::new(Log::default()));

        let users = use_case(&log, false)
            .list_all_users(&ListAllUsers)
            .await
            .expect("list succeeds");

        assert_eq!(calls(&log), 1);
        assert_eq!(
            users
                .iter()
                .map(|u| u.username.as_str())
                .collect::<Vec<_>>(),
            vec!["alice", "bob"],
            "the service must not re-sort or filter the query result"
        );
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .list_all_users(&ListAllUsers)
            .await
            .expect_err("select failure propagates");

        assert_eq!(error.to_string(), "simulated select failure");
    }
}
