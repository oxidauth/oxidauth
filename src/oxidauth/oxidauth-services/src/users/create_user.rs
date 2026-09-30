use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, users::create_user::*};
use oxidauth_repository::users::insert_user::InsertUserQuery;

#[derive(Clone)]
pub struct CreateUserUseCase<T>
where
    T: InsertUserQuery,
{
    users: T,
}

impl<T> CreateUserUseCase<T>
where
    T: InsertUserQuery,
{
    pub fn new(users: T) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<T> CreateUserServiceTrait for CreateUserUseCase<T>
where
    T: InsertUserQuery,
{
    #[tracing::instrument(name = "CreateUserUseCase::create_user", skip(self))]
    async fn create_user(&self, req: &CreateUser) -> Result<User, BoxedError> {
        let user = self
            .users
            .insert_user(req)
            .await?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::users::UserStatus;
    use serde_json::json;

    use super::*;

    fn stored_user() -> User {
        User {
            id: uuid::uuid!("11111111-1111-4111-8111-111111111111"),
            kind: UserKind::Human,
            status: UserStatus::Enabled,
            username: "octocat".to_owned(),
            email: Some("octocat@example.com".to_owned()),
            first_name: None,
            last_name: None,
            profile: json!({ "team": "core" }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Snapshot of what the use case hands to the insert query.
    #[derive(Clone, Debug, PartialEq)]
    struct CapturedInsert {
        id: Option<uuid::Uuid>,
        kind: Option<UserKind>,
        status: Option<&'static str>,
        username: String,
        email: Option<String>,
        profile: Option<serde_json::Value>,
    }

    #[derive(Default)]
    struct Log {
        inserts: Vec<CapturedInsert>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn inserts(log: &SharedLog) -> Vec<String> {
        // `CapturedInsert` has no cheap summary; tests assert the full
        // capture, this helper only checks call count/username.
        let log = log.lock().expect("log");
        log.inserts
            .iter()
            .map(|c| c.username.clone())
            .collect()
    }

    fn captured(log: &SharedLog) -> Vec<CapturedInsert> {
        log.lock()
            .expect("log")
            .inserts
            .clone()
    }

    struct MockUsers {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl InsertUserQuery for MockUsers {
        async fn insert_user(&self, req: &CreateUser) -> Result<User, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .inserts
                .push(CapturedInsert {
                    id: req.id,
                    kind: req.kind.clone(),
                    status: req
                        .status
                        .as_ref()
                        .map(|status| status.into()),
                    username: req.username.clone(),
                    email: req.email.clone(),
                    profile: req.profile.clone(),
                });

            if self.fail {
                return Err("simulated insert failure".into());
            }

            Ok(stored_user())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> CreateUserUseCase<MockUsers> {
        CreateUserUseCase::new(MockUsers {
            log: log.clone(),
            fail,
        })
    }

    fn base_request() -> CreateUser {
        CreateUser {
            id: None,
            kind: Some(UserKind::Api),
            status: Some(UserStatus::Invited),
            username: "ci-bot".to_owned(),
            email: Some("bot@example.com".to_owned()),
            first_name: None,
            last_name: None,
            profile: Some(json!({ "scopes": ["ci"] })),
        }
    }

    // Pure delegate: the service is `self.users.insert_user(req).await?` with no
    // branching, so the verbatim-passthrough and error tests are the whole
    // contract.
    #[tokio::test]
    async fn request_reaches_the_repository_verbatim_and_response_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));

        let user = use_case(&log, false)
            .create_user(&base_request())
            .await
            .expect("create succeeds");

        let captured = captured(&log);
        assert_eq!(captured.len(), 1);
        assert_eq!(
            captured[0],
            CapturedInsert {
                id: None,
                kind: Some(UserKind::Api),
                status: Some("invited"),
                username: "ci-bot".to_owned(),
                email: Some("bot@example.com".to_owned()),
                profile: Some(json!({ "scopes": ["ci"] })),
            },
            "the request must reach the insert query untouched"
        );
        assert_eq!(user.username, "octocat");
        assert_eq!(user.email.as_deref(), Some("octocat@example.com"));
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .create_user(&base_request())
            .await
            .expect_err("insert failure propagates");

        assert_eq!(error.to_string(), "simulated insert failure");
        assert_eq!(inserts(&log), vec!["ci-bot".to_owned()]);
    }
}
