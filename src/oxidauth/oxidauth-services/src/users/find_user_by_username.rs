use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    users::{UserNotFoundError, find_user_by_username::*},
};
use oxidauth_repository::users::select_user_by_username_query::SelectUserByUsernameQuery;

pub struct FindUserByUsernameUseCase<T>
where
    T: SelectUserByUsernameQuery,
{
    users: T,
}

impl<T> FindUserByUsernameUseCase<T>
where
    T: SelectUserByUsernameQuery,
{
    pub fn new(users: T) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<T> FindUserByUsernameServiceTrait for FindUserByUsernameUseCase<T>
where
    T: SelectUserByUsernameQuery,
{
    #[tracing::instrument(name = "FindUserByUsernameUseCase::find_user_by_username", skip(self))]
    async fn find_user_by_username(&self, params: &FindUserByUsername) -> Result<User, BoxedError> {
        let user = self
            .users
            .select_user_by_username(&params.username)
            .await?
            .ok_or_else(|| UserNotFoundError::username(&params.username))?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::users::{UserKind, UserStatus, Username};
    use serde_json::json;

    use super::*;

    fn stored_user() -> User {
        User {
            id: uuid::uuid!("11111111-1111-4111-8111-111111111111"),
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
        lookups: Vec<String>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn lookups(log: &SharedLog) -> Vec<String> {
        log.lock()
            .expect("log")
            .lookups
            .clone()
    }

    struct MockUsers {
        log: SharedLog,
        /// `Ok(None)` (no row) vs `Err(..)` — the two failure modes the
        /// service must keep distinct.
        missing: bool,
        fail: bool,
    }

    #[async_trait]
    impl SelectUserByUsernameQuery for MockUsers {
        async fn select_user_by_username(
            &self,
            req: &Username,
        ) -> Result<Option<User>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .lookups
                .push(req.to_string());

            if self.fail {
                return Err("simulated select failure".into());
            }

            if self.missing {
                return Ok(None);
            }

            Ok(Some(stored_user()))
        }
    }

    fn use_case(
        log: &SharedLog,
        missing: bool,
        fail: bool,
    ) -> FindUserByUsernameUseCase<MockUsers> {
        FindUserByUsernameUseCase::new(MockUsers {
            log: log.clone(),
            missing,
            fail,
        })
    }

    #[tokio::test]
    async fn username_reaches_the_query_and_the_row_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));

        let user = use_case(&log, false, false)
            .find_user_by_username(&FindUserByUsername {
                username: "octocat"
                    .parse()
                    .expect("username"),
            })
            .await
            .expect("find succeeds");

        assert_eq!(lookups(&log), vec!["octocat".to_owned()]);
        assert_eq!(user.username, "octocat");
    }

    #[tokio::test]
    async fn missing_row_becomes_a_user_not_found_error() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true, false)
            .find_user_by_username(&FindUserByUsername {
                username: "ghost"
                    .parse()
                    .expect("username"),
            })
            .await
            .expect_err("Ok(None) is mapped to a not-found error");

        assert_eq!(error.to_string(), "user not found where: username == ghost");
        // the typed error must survive for callers that downcast (auth
        // flows branch on it, not on Display). Note the payload is
        // `Box<UserNotFoundError>`: `ok_or_else(...)?` feeds `From` with the
        // already-boxed error (std's blanket `From<E> for Box<dyn Error>`
        // keeps `E = Box<T>` as payload), unlike a direct `Err(box)` return
        // which coerces and stays unboxed
        assert!(
            error
                .downcast_ref::<Box<UserNotFoundError>>()
                .is_some()
        );
        assert_eq!(lookups(&log), vec!["ghost".to_owned()]);
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped_not_as_not_found() {
        // the query error must stay distinguishable from a missing row —
        // callers (auth flows) branch on this
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, false, true)
            .find_user_by_username(&FindUserByUsername {
                username: "octocat"
                    .parse()
                    .expect("username"),
            })
            .await
            .expect_err("select failure propagates verbatim");

        assert_eq!(error.to_string(), "simulated select failure");
        assert!(
            !error
                .to_string()
                .contains("not found")
        );
    }
}
