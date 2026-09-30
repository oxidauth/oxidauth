use std::ops::Add;

use async_trait::async_trait;
use chrono::{Days, Utc};
use oxidauth_kernel::{
    error::BoxedError,
    invitations::create_invitation::{
        CreateInvitationParams,
        CreateInvitationResponse,
        CreateInvitationServiceTrait,
    },
    users::create_user::CreateUserServiceTrait,
};
use oxidauth_repository::invitations::insert_invitation::{
    InsertInvitationParams,
    InsertInvitationQuery,
};

pub struct CreateInvitationUseCase<T, U>
where
    T: InsertInvitationQuery,
    U: CreateUserServiceTrait,
{
    insert_invitation: T,
    create_user: U,
}

impl<T, U> CreateInvitationUseCase<T, U>
where
    T: InsertInvitationQuery,
    U: CreateUserServiceTrait,
{
    pub fn new(insert_invitation: T, create_user: U) -> Self {
        Self {
            insert_invitation,
            create_user,
        }
    }
}

#[async_trait]
impl<T, U> CreateInvitationServiceTrait for CreateInvitationUseCase<T, U>
where
    T: InsertInvitationQuery,
    U: CreateUserServiceTrait,
{
    #[tracing::instrument(name = "CreateInvitationUseCase::create_invitation", skip(self))]
    async fn create_invitation(
        &self,
        params: &CreateInvitationParams,
    ) -> Result<CreateInvitationResponse, BoxedError> {
        let user = self
            .create_user
            .create_user(&params.user)
            .await?;

        let insert_invitation_params = InsertInvitationParams {
            id: params.id,
            user_id: user.id,
            // TODO(dewey4iv): https://www.pivotaltracker.com/story/show/186949366
            expires_at: params
                .expires_at
                .unwrap_or_else(|| Utc::now().add(Days::new(7))),
        };

        let invitation = self
            .insert_invitation
            .insert_invitation(&insert_invitation_params)
            .await?;

        Ok(CreateInvitationResponse { invitation, user })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Days;
    use oxidauth_kernel::{
        invitations::Invitation,
        users::{
            User,
            create_user::{CreateUser, CreateUserServiceTrait},
        },
    };
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn user_id() -> Uuid {
        uuid::uuid!("77777777-7777-4777-8777-777777777777")
    }

    fn invitation_id() -> Uuid {
        uuid::uuid!("88888888-8888-4888-8888-888888888888")
    }

    #[derive(Clone, Debug)]
    struct CapturedInsert {
        id: Option<Uuid>,
        user_id: Uuid,
        expires_at: chrono::DateTime<Utc>,
    }

    #[derive(Default)]
    struct Log {
        user_creates: usize,
        inserts: Vec<CapturedInsert>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn user_creates(log: &SharedLog) -> usize {
        log.lock()
            .expect("log")
            .user_creates
    }

    fn inserts(log: &SharedLog) -> Vec<CapturedInsert> {
        log.lock()
            .expect("log")
            .inserts
            .clone()
    }

    struct MockCreateUser {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl CreateUserServiceTrait for MockCreateUser {
        async fn create_user(&self, params: &CreateUser) -> Result<User, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .user_creates += 1;

            if self.fail {
                return Err("simulated user creation failure".into());
            }

            Ok(User {
                id: params
                    .id
                    .unwrap_or_else(user_id),
                kind: Default::default(),
                status: Default::default(),
                username: params.username.clone(),
                email: params.email.clone(),
                first_name: params.first_name.clone(),
                last_name: params.last_name.clone(),
                profile: params
                    .profile
                    .clone()
                    .unwrap_or_else(|| json!({})),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    struct MockInsertInvitation {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl InsertInvitationQuery for MockInsertInvitation {
        async fn insert_invitation(
            &self,
            req: &InsertInvitationParams,
        ) -> Result<Invitation, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .inserts
                .push(CapturedInsert {
                    id: req.id,
                    user_id: req.user_id,
                    expires_at: req.expires_at,
                });

            if self.fail {
                return Err("simulated insert failure".into());
            }

            Ok(Invitation {
                id: req
                    .id
                    .unwrap_or_else(invitation_id),
                user_id: req.user_id,
                expires_at: req.expires_at,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    fn use_case(
        log: &SharedLog,
        create_user_fails: bool,
        insert_fails: bool,
    ) -> CreateInvitationUseCase<MockInsertInvitation, MockCreateUser> {
        CreateInvitationUseCase::new(
            MockInsertInvitation {
                log: log.clone(),
                fail: insert_fails,
            },
            MockCreateUser {
                log: log.clone(),
                fail: create_user_fails,
            },
        )
    }

    fn params() -> CreateInvitationParams {
        CreateInvitationParams {
            id: None,
            expires_at: None,
            user: CreateUser {
                id: None,
                kind: None,
                status: None,
                username: "invited-user".to_owned(),
                email: Some("invited@example.com".to_owned()),
                first_name: None,
                last_name: None,
                profile: None,
            },
        }
    }

    #[tokio::test]
    async fn explicit_id_and_expiry_are_passed_through_and_user_id_comes_from_created_user() {
        // The invitation model has no separate token column — the invitation
        // `id` itself is the bearer token; `create_invitation` only ever sets
        // (id, user_id, expires_at), everything else is DB-managed.
        let log = Arc::new(Mutex::new(Log::default()));
        let fixed_expiry = Utc::now()
            .naive_utc()
            .date()
            .and_hms_opt(12, 0, 0)
            .expect("time")
            .and_utc();
        let mut params = params();
        params.id = Some(invitation_id());
        params.expires_at = Some(fixed_expiry);

        let response = use_case(&log, false, false)
            .create_invitation(&params)
            .await
            .expect("invitation created");

        let captured = inserts(&log);
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].id, Some(invitation_id()));
        assert_eq!(
            captured[0].user_id,
            user_id(),
            "user_id is the id of the just-created user, not a request field"
        );
        assert_eq!(captured[0].expires_at, fixed_expiry);

        assert_eq!(response.invitation.id, invitation_id());
        assert_eq!(response.user.id, user_id());
        assert_eq!(response.user.username, "invited-user");
    }

    #[tokio::test]
    async fn omitted_expiry_defaults_to_seven_days_from_now() {
        let log = Arc::new(Mutex::new(Log::default()));
        let params = params();

        let before = Utc::now();
        use_case(&log, false, false)
            .create_invitation(&params)
            .await
            .expect("invitation created");
        let after = Utc::now();

        let captured = inserts(&log);
        let expiry = captured[0].expires_at;

        assert!(
            expiry >= before.add(Days::new(7)) && expiry <= after.add(Days::new(7)),
            "default expiry is now + 7 days (got {expiry})"
        );
    }

    #[tokio::test]
    async fn omitted_id_is_forwarded_as_none_for_the_database_to_assign() {
        let log = Arc::new(Mutex::new(Log::default()));
        let params = params();

        use_case(&log, false, false)
            .create_invitation(&params)
            .await
            .expect("invitation created");

        assert_eq!(inserts(&log)[0].id, None);
    }

    #[tokio::test]
    async fn user_creation_failure_aborts_before_any_insert() {
        let log = Arc::new(Mutex::new(Log::default()));
        let params = params();

        let error = use_case(&log, true, false)
            .create_invitation(&params)
            .await
            .expect_err("user creation failure surfaces");

        assert_eq!(error.to_string(), "simulated user creation failure");
        assert_eq!(user_creates(&log), 1);
        assert!(inserts(&log).is_empty());
    }

    #[tokio::test]
    async fn insert_failure_propagates_after_the_user_was_created() {
        let log = Arc::new(Mutex::new(Log::default()));
        let params = params();

        let error = use_case(&log, false, true)
            .create_invitation(&params)
            .await
            .expect_err("insert failure surfaces");

        assert_eq!(error.to_string(), "simulated insert failure");
        assert_eq!(user_creates(&log), 1);
        assert_eq!(inserts(&log).len(), 1);
    }
}
