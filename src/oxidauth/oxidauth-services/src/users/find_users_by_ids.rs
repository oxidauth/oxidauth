use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, users::find_users_by_ids::*};
use oxidauth_repository::users::select_users_by_ids_query::SelectUsersByIdsQuery;

pub struct FindUsersByIdsUseCase<T>
where
    T: SelectUsersByIdsQuery,
{
    users: T,
}

impl<T> FindUsersByIdsUseCase<T>
where
    T: SelectUsersByIdsQuery,
{
    pub fn new(users: T) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<T> FindUsersByIdsServiceTrait for FindUsersByIdsUseCase<T>
where
    T: SelectUsersByIdsQuery,
{
    #[tracing::instrument(name = "FindUsersByIdsUseCase::find_users_by_ids", skip(self))]
    async fn find_users_by_ids(&self, req: &FindUsersByIds) -> Result<UsersByIds, BoxedError> {
        self.users
            .select_users_by_ids(req)
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

    fn requested_ids() -> Vec<Uuid> {
        vec![
            uuid::uuid!("11111111-1111-4111-8111-111111111111"),
            uuid::uuid!("22222222-2222-4222-8222-222222222222"),
        ]
    }

    fn user(id: Uuid) -> User {
        User {
            id,
            kind: UserKind::Human,
            status: UserStatus::Enabled,
            username: format!("user-{id}"),
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
        calls: Vec<Vec<Uuid>>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn calls(log: &SharedLog) -> Vec<Vec<Uuid>> {
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
    impl SelectUsersByIdsQuery for MockUsers {
        async fn select_users_by_ids(
            &self,
            req: &FindUsersByIds,
        ) -> Result<UsersByIds, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .calls
                .push(req.user_ids.clone());

            if self.fail {
                return Err("simulated select failure".into());
            }

            // partial match: first id found, second missing
            let mut ids = req.user_ids.iter().copied();
            let found = ids
                .next()
                .map(user)
                .into_iter()
                .collect();
            let missing = ids.collect();

            Ok(UsersByIds {
                users: found,
                user_ids_not_found: missing,
            })
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> FindUsersByIdsUseCase<MockUsers> {
        FindUsersByIdsUseCase::new(MockUsers {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate — but the partial-match envelope (`user_ids_not_found`)
    // must survive the service layer untouched.
    #[tokio::test]
    async fn id_list_reaches_the_query_and_the_partial_match_envelope_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));
        let ids = requested_ids();

        let result = use_case(&log, false)
            .find_users_by_ids(&FindUsersByIds {
                user_ids: ids.clone(),
            })
            .await
            .expect("find succeeds");

        assert_eq!(calls(&log), vec![ids.clone()]);
        assert_eq!(result.users.len(), 1);
        assert_eq!(result.users[0].id, ids[0]);
        assert_eq!(
            result.user_ids_not_found,
            vec![ids[1]],
            "not-found ids must be relayed verbatim, not re-derived"
        );
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .find_users_by_ids(&FindUsersByIds {
                user_ids: requested_ids(),
            })
            .await
            .expect_err("select failure propagates");

        assert_eq!(error.to_string(), "simulated select failure");
    }
}
