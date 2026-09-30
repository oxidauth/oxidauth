use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    users::{find_user_by_id::FindUserById, update_user::*},
};
use oxidauth_repository::users::{
    select_user_by_id_query::SelectUserByIdQuery,
    update_user::UpdateUserQuery,
};

pub struct UpdateUserUseCase<S, U>
where
    S: SelectUserByIdQuery,
    U: UpdateUserQuery,
{
    user_by_id: S,
    update_user: U,
}

impl<S, U> UpdateUserUseCase<S, U>
where
    S: SelectUserByIdQuery,
    U: UpdateUserQuery,
{
    pub fn new(user_by_id: S, update_user: U) -> Self {
        Self {
            user_by_id,
            update_user,
        }
    }
}

#[async_trait]
impl<S, U> UpdateUserServiceTrait for UpdateUserUseCase<S, U>
where
    S: SelectUserByIdQuery,
    U: UpdateUserQuery,
{
    #[tracing::instrument(name = "UpdateUserUseCase::update_user", skip(self))]
    async fn update_user(&self, params: &mut UpdateUser) -> Result<User, BoxedError> {
        let current = self
            .user_by_id
            .select_user_by_id(&FindUserById { user_id: params.id })
            .await?;

        if params.username.is_none() {
            params.username = Some(current.username);
        }

        if params.email.is_none() {
            params.email = current.email;
        }

        if params.status.is_none() {
            params.status = Some(current.status);
        }

        if params.profile.is_none() {
            params.profile = Some(current.profile);
        }

        self.update_user
            .update_user(params)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::users::UserKind;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn user_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    /// The *current* (pre-update) users row as the select query returns it.
    fn current_user() -> User {
        User {
            id: user_id(),
            kind: UserKind::Human,
            status: UserStatus::Disabled,
            username: "octocat".to_owned(),
            email: Some("octocat@example.com".to_owned()),
            first_name: Some("Octavia".to_owned()),
            last_name: Some("Cat".to_owned()),
            profile: json!({ "team": "core" }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Snapshot of what the use case finally hands to the update query.
    #[derive(Debug, Clone, PartialEq)]
    struct CapturedUpdate {
        id: Uuid,
        username: Option<String>,
        email: Option<String>,
        first_name: Option<String>,
        last_name: Option<String>,
        status: Option<&'static str>,
        profile: Option<serde_json::Value>,
    }

    #[derive(Default)]
    struct Log {
        finds: Vec<Uuid>,
        updates: Vec<CapturedUpdate>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn finds(log: &SharedLog) -> Vec<Uuid> {
        log.lock()
            .expect("log")
            .finds
            .clone()
    }

    fn updates(log: &SharedLog) -> Vec<CapturedUpdate> {
        log.lock()
            .expect("log")
            .updates
            .clone()
    }

    struct MockFind {
        log: SharedLog,
        current_email: bool,
        fail: bool,
    }

    #[async_trait]
    impl SelectUserByIdQuery for MockFind {
        async fn select_user_by_id(&self, req: &FindUserById) -> Result<User, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .finds
                .push(req.user_id);

            if self.fail {
                return Err("simulated lookup failure".into());
            }

            let mut current = current_user();
            if !self.current_email {
                current.email = None;
            }

            Ok(current)
        }
    }

    struct MockUpdate {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl UpdateUserQuery for MockUpdate {
        async fn update_user(&self, req: &UpdateUser) -> Result<User, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .updates
                .push(CapturedUpdate {
                    id: req.id,
                    username: req.username.clone(),
                    email: req.email.clone(),
                    first_name: req.first_name.clone(),
                    last_name: req.last_name.clone(),
                    status: req
                        .status
                        .as_ref()
                        .map(|status| status.into()),
                    profile: req.profile.clone(),
                });

            if self.fail {
                return Err("simulated update failure".into());
            }

            Ok(current_user())
        }
    }

    fn use_case(
        log: &SharedLog,
        current_email: bool,
        find_fails: bool,
        update_fails: bool,
    ) -> UpdateUserUseCase<MockFind, MockUpdate> {
        UpdateUserUseCase::new(
            MockFind {
                log: log.clone(),
                current_email,
                fail: find_fails,
            },
            MockUpdate {
                log: log.clone(),
                fail: update_fails,
            },
        )
    }

    fn base_request() -> UpdateUser {
        UpdateUser {
            id: user_id(),
            username: None,
            email: None,
            first_name: None,
            last_name: None,
            status: None,
            profile: None,
        }
    }

    #[tokio::test]
    async fn omitted_fields_are_backfilled_from_the_current_row() {
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();

        let user = use_case(&log, true, false, false)
            .update_user(&mut req)
            .await
            .expect("update succeeds");

        assert_eq!(finds(&log), vec![user_id()]);
        let captured = updates(&log);
        assert_eq!(captured.len(), 1);
        assert_eq!(
            captured[0],
            CapturedUpdate {
                id: user_id(),
                username: Some("octocat".to_owned()),
                email: Some("octocat@example.com".to_owned()),
                // BUG(pinned): username/email/status/profile are backfilled
                // from the current row, but first/last name are NOT — and
                // `update_user.sql` overwrites every column, so an omitted
                // name silently wipes the stored value to NULL.
                first_name: None,
                last_name: None,
                status: Some("disabled"),
                profile: Some(json!({ "team": "core" })),
            }
        );
        assert_eq!(user.username, "octocat");
    }

    #[tokio::test]
    async fn supplied_fields_win_over_the_current_row() {
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();
        req.username = Some("renamed".to_owned());
        req.status = Some(UserStatus::Enabled);
        req.first_name = Some("New".to_owned());

        use_case(&log, true, false, false)
            .update_user(&mut req)
            .await
            .expect("update succeeds");

        let captured = updates(&log);
        assert_eq!(captured.len(), 1);
        assert_eq!(
            captured[0]
                .username
                .as_deref(),
            Some("renamed")
        );
        assert_eq!(captured[0].status, Some("enabled"));
        assert_eq!(
            captured[0]
                .first_name
                .as_deref(),
            Some("New")
        );
        // untouched fields still come from the current row
        assert_eq!(captured[0].email.as_deref(), Some("octocat@example.com"));
        assert_eq!(captured[0].profile, Some(json!({ "team": "core" })));
    }

    #[tokio::test]
    async fn omitted_email_on_a_row_without_email_stays_absent() {
        // email is merged with `params.email = current.email` (not
        // `Some(current.email)`) — when the row has no email there is
        // nothing to backfill and the omission reaches the repository as
        // `None`.
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();

        use_case(&log, false, false, false)
            .update_user(&mut req)
            .await
            .expect("update succeeds");

        let captured = updates(&log);
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].email, None);
        assert_eq!(
            captured[0]
                .username
                .as_deref(),
            Some("octocat")
        );
    }

    #[tokio::test]
    async fn lookup_error_propagates_without_calling_update() {
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();

        let error = use_case(&log, true, true, false)
            .update_user(&mut req)
            .await
            .expect_err("lookup failure propagates");

        assert_eq!(error.to_string(), "simulated lookup failure");
        assert!(updates(&log).is_empty(), "no update after a failed find");
    }

    #[tokio::test]
    async fn update_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();

        let error = use_case(&log, true, false, true)
            .update_user(&mut req)
            .await
            .expect_err("update failure propagates");

        assert_eq!(error.to_string(), "simulated update failure");
        assert_eq!(updates(&log).len(), 1);
    }
}
