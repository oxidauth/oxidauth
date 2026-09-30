use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    invitations::{
        Invitation,
        accept_invitation::{AcceptInvitationParams, AcceptInvitationServiceTrait},
        delete_invitation::DeleteInvitationParams,
    },
    user_authorities::create_user_authority::{
        CreateUserAuthorityParams,
        CreateUserAuthorityService,
    },
    users::{
        User,
        update_user::{UpdateUser, UpdateUserService},
    },
};
use oxidauth_repository::invitations::delete_invitation_by_id::DeleteInvitationByIdQuery;

pub struct AcceptInvitationUseCase<I>
where
    I: DeleteInvitationByIdQuery,
{
    update_user: UpdateUserService,
    user_authority: CreateUserAuthorityService,
    delete_invitation: I,
}

impl<I> AcceptInvitationUseCase<I>
where
    I: DeleteInvitationByIdQuery,
{
    pub fn new(
        update_user: UpdateUserService,
        user_authority: CreateUserAuthorityService,
        delete_invitation: I,
    ) -> Self {
        Self {
            update_user,
            user_authority,
            delete_invitation,
        }
    }
}

#[async_trait]
impl<I> AcceptInvitationServiceTrait for AcceptInvitationUseCase<I>
where
    I: DeleteInvitationByIdQuery,
{
    #[tracing::instrument(name = "AcceptInvitationUseCase::accept_invitation", skip(self))]
    async fn accept_invitation(&self, params: &AcceptInvitationParams) -> Result<User, BoxedError> {
        let delete_invitation = DeleteInvitationParams {
            id: params.invitation_id,
        };

        let Invitation { user_id, .. } = self
            .delete_invitation
            .delete_invitation_by_id(&delete_invitation)
            .await?;

        let create_user_authority = CreateUserAuthorityParams {
            user_id,
            client_key: params
                .user_authority
                .client_key,
            params: params
                .user_authority
                .params
                .clone(),
        };

        let _user_authority = self
            .user_authority
            .create_user_authority(&create_user_authority)
            .await?;

        let mut update_user: UpdateUser = (user_id, &params.user).into();

        let user = self
            .update_user
            .update_user(&mut update_user)
            .await?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::{DateTime, Utc};
    use oxidauth_kernel::{
        JsonValue,
        auth::register::RegisterParams,
        invitations::accept_invitation::AcceptInvitationUserParams,
        user_authorities::{UserAuthority, create_user_authority::CreateUserAuthorityServiceTrait},
        users::update_user::UpdateUserServiceTrait,
    };
    use serde_json::{Value, json};
    use uuid::Uuid;

    use super::*;

    fn invitation_id() -> Uuid {
        uuid::uuid!("99999999-9999-4999-8999-999999999999")
    }

    fn invited_user_id() -> Uuid {
        uuid::uuid!("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa")
    }

    fn authority_client_key() -> Uuid {
        uuid::uuid!("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb")
    }

    fn invitation(expires_at: DateTime<Utc>) -> Invitation {
        Invitation {
            id: invitation_id(),
            user_id: invited_user_id(),
            expires_at,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Snapshot of the UpdateUser handed to the user service.
    #[derive(Clone, Debug, PartialEq)]
    struct CapturedUpdate {
        id: Uuid,
        username: Option<String>,
        email: Option<String>,
        first_name: Option<String>,
        last_name: Option<String>,
        status: Option<String>,
        profile: Option<Value>,
    }

    #[derive(Clone, Debug)]
    struct CapturedUserAuthority {
        user_id: Uuid,
        client_key: Uuid,
        params: Value,
    }

    #[derive(Default)]
    struct Log {
        steps: Vec<String>,
        deletes: Vec<Uuid>,
        user_authorities: Vec<CapturedUserAuthority>,
        updates: Vec<CapturedUpdate>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn record(log: &SharedLog, step: &str) {
        log.lock()
            .expect("log")
            .steps
            .push(step.to_owned());
    }

    fn steps(log: &SharedLog) -> Vec<String> {
        log.lock()
            .expect("log")
            .steps
            .clone()
    }

    fn deletes(log: &SharedLog) -> Vec<Uuid> {
        log.lock()
            .expect("log")
            .deletes
            .clone()
    }

    fn user_authorities(log: &SharedLog) -> Vec<CapturedUserAuthority> {
        log.lock()
            .expect("log")
            .user_authorities
            .clone()
    }

    fn updates(log: &SharedLog) -> Vec<CapturedUpdate> {
        log.lock()
            .expect("log")
            .updates
            .clone()
    }

    struct MockDeleteInvitation {
        log: SharedLog,
        /// `None` = row missing (RowNotFound); otherwise the invitation
        /// returned by `DELETE ... RETURNING *`.
        stored: Option<Invitation>,
    }

    #[async_trait]
    impl DeleteInvitationByIdQuery for MockDeleteInvitation {
        async fn delete_invitation_by_id(
            &self,
            req: &DeleteInvitationParams,
        ) -> Result<Invitation, BoxedError> {
            record(&self.log, "delete");
            self.log
                .lock()
                .expect("log")
                .deletes
                .push(req.id);

            match &self.stored {
                Some(invitation) => {
                    Ok(Invitation {
                        id: invitation.id,
                        user_id: invitation.user_id,
                        expires_at: invitation.expires_at,
                        created_at: invitation.created_at,
                        updated_at: invitation.updated_at,
                    })
                },
                None => Err("RowNotFound".into()),
            }
        }
    }

    struct MockCreateUserAuthority {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl CreateUserAuthorityServiceTrait for MockCreateUserAuthority {
        async fn create_user_authority(
            &self,
            params: &CreateUserAuthorityParams,
        ) -> Result<UserAuthority, BoxedError> {
            record(&self.log, "user_authority");
            self.log
                .lock()
                .expect("log")
                .user_authorities
                .push(CapturedUserAuthority {
                    user_id: params.user_id,
                    client_key: params.client_key,
                    params: params
                        .params
                        .clone()
                        .inner_value(),
                });

            if self.fail {
                return Err("simulated user authority creation failure".into());
            }

            Ok(UserAuthority {
                user_id: params.user_id,
                authority_id: Uuid::new_v4(),
                user_identifier: "invited-user".to_owned(),
                params: params.params.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    struct MockUpdateUser {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl UpdateUserServiceTrait for MockUpdateUser {
        async fn update_user(&self, params: &mut UpdateUser) -> Result<User, BoxedError> {
            record(&self.log, "update");
            self.log
                .lock()
                .expect("log")
                .updates
                .push(CapturedUpdate {
                    id: params.id,
                    username: params.username.clone(),
                    email: params.email.clone(),
                    first_name: params.first_name.clone(),
                    last_name: params.last_name.clone(),
                    status: params
                        .status
                        .as_ref()
                        .map(|status| format!("{status:?}")),
                    profile: params.profile.clone(),
                });

            if self.fail {
                return Err("simulated user update failure".into());
            }

            Ok(User {
                id: params.id,
                kind: Default::default(),
                status: Default::default(),
                username: params
                    .username
                    .clone()
                    .unwrap_or_else(|| "invited-user".to_owned()),
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

    fn use_case(
        log: &SharedLog,
        stored: Option<Invitation>,
        user_authority_fails: bool,
        update_fails: bool,
    ) -> AcceptInvitationUseCase<MockDeleteInvitation> {
        AcceptInvitationUseCase::new(
            Arc::new(MockUpdateUser {
                log: log.clone(),
                fail: update_fails,
            }),
            Arc::new(MockCreateUserAuthority {
                log: log.clone(),
                fail: user_authority_fails,
            }),
            MockDeleteInvitation {
                log: log.clone(),
                stored,
            },
        )
    }

    fn params() -> AcceptInvitationParams {
        AcceptInvitationParams {
            invitation_id: invitation_id(),
            user: AcceptInvitationUserParams {
                username: "claimed-username".to_owned(),
                email: Some("claimed@example.com".to_owned()),
                first_name: Some("Ada".to_owned()),
                last_name: Some("Lovelace".to_owned()),
                // OXA-000009: `AcceptInvitationUserParams` has no `status` field —
                // an invitee cannot pre-set their own account status.
                profile: Some(json!({ "bio": "analyst" })),
            },
            user_authority: RegisterParams {
                client_key: authority_client_key(),
                params: JsonValue::new(json!({ "password": "s3cret" })),
            },
        }
    }

    #[tokio::test]
    async fn accept_deletes_the_invitation_then_creates_authority_and_updates_user() {
        let log = Arc::new(Mutex::new(Log::default()));
        let stored = invitation(Utc::now() + chrono::Duration::days(7));

        let user = use_case(&log, Some(stored), false, false)
            .accept_invitation(&params())
            .await
            .expect("accept succeeds");

        assert_eq!(steps(&log), vec!["delete", "user_authority", "update"]);
        assert_eq!(deletes(&log), vec![invitation_id()]);

        let authority = &user_authorities(&log)[0];
        assert_eq!(authority.user_id, invited_user_id());
        assert_eq!(authority.client_key, authority_client_key());
        assert_eq!(authority.params, json!({ "password": "s3cret" }));

        let update = &updates(&log)[0];
        assert_eq!(
            update.id,
            invited_user_id(),
            "the invited user (from the invitation row) is the update target"
        );
        assert_eq!(update.username.as_deref(), Some("claimed-username"));
        assert_eq!(update.email.as_deref(), Some("claimed@example.com"));
        assert_eq!(
            update.status, None,
            "OXA-000009 (flipped pin): the invitation flow must never carry a \
             status into `UpdateUser` — the stored value is backfilled instead"
        );
        assert_eq!(update.profile, Some(json!({ "bio": "analyst" })));

        assert_eq!(user.username, "claimed-username");
    }

    #[tokio::test]
    async fn a_request_body_still_carrying_status_is_deserialised_but_never_honoured() {
        // OXA-000009: serde is permissive (no `deny_unknown_fields` in the
        // workspace), so a legacy invitee that still POSTs `"status":
        // "enabled"` about themselves keeps deserialising — but the captured
        // `UpdateUser` must carry `status: None`, making the self re-enable
        // attempt a no-op.
        let log = Arc::new(Mutex::new(Log::default()));
        let stored = invitation(Utc::now() + chrono::Duration::days(7));

        let user: AcceptInvitationUserParams = serde_json::from_value(json!({
            "username": "claimed-username",
            "email": "claimed@example.com",
            "first_name": "Ada",
            "last_name": "Lovelace",
            "status": "enabled",
            "profile": { "bio": "analyst" },
        }))
        .expect("legacy bodies carrying `status` must still deserialise");

        let params = AcceptInvitationParams {
            invitation_id: invitation_id(),
            user,
            user_authority: RegisterParams {
                client_key: authority_client_key(),
                params: JsonValue::new(json!({ "password": "s3cret" })),
            },
        };

        use_case(&log, Some(stored), false, false)
            .accept_invitation(&params)
            .await
            .expect("accept succeeds");

        assert_eq!(
            updates(&log)[0].status,
            None,
            "a self-supplied status is never honoured, whatever the body says"
        );
    }

    #[tokio::test]
    async fn expired_invitation_is_still_accepted() {
        // BUG(pinned): expired invitations are accepted — neither
        // accept_invitation.rs nor the `DELETE ... RETURNING *` it runs
        // (delete_invitation_by_id.sql) ever compares `expires_at` to now,
        // so an expired token remains usable forever. The service happily
        // accepts an invitation that expired a week ago.
        let log = Arc::new(Mutex::new(Log::default()));
        let expired = invitation(Utc::now() - chrono::Duration::weeks(1));

        let user = use_case(&log, Some(expired), false, false)
            .accept_invitation(&params())
            .await
            .expect("expired invitations currently pass — no expiry check exists");

        assert_eq!(steps(&log), vec!["delete", "user_authority", "update"]);
        assert_eq!(user.username, "claimed-username");
    }

    #[tokio::test]
    async fn missing_invitation_surfaces_the_repository_error_without_side_effects() {
        // There is no "accepted" column in the invitations schema and no
        // accepted-state in the service: accepting deletes the row, so an
        // already-accepted invitation re-acceptance lands on this same
        // RowNotFound path (second accept = missing row).
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, None, false, false)
            .accept_invitation(&params())
            .await
            .expect_err("missing invitation must fail");

        assert_eq!(error.to_string(), "RowNotFound");
        assert_eq!(steps(&log), vec!["delete"]);
        assert!(user_authorities(&log).is_empty());
        assert!(updates(&log).is_empty());
    }

    #[tokio::test]
    async fn user_authority_failure_aborts_before_the_user_update() {
        let log = Arc::new(Mutex::new(Log::default()));
        let stored = invitation(Utc::now() + chrono::Duration::days(7));

        let error = use_case(&log, Some(stored), true, false)
            .accept_invitation(&params())
            .await
            .expect_err("authority failure surfaces");

        assert_eq!(
            error.to_string(),
            "simulated user authority creation failure"
        );
        assert_eq!(steps(&log), vec!["delete", "user_authority"]);
        assert!(updates(&log).is_empty());
    }

    #[tokio::test]
    async fn user_update_failure_propagates() {
        let log = Arc::new(Mutex::new(Log::default()));
        let stored = invitation(Utc::now() + chrono::Duration::days(7));

        let error = use_case(&log, Some(stored), false, true)
            .accept_invitation(&params())
            .await
            .expect_err("update failure surfaces");

        assert_eq!(error.to_string(), "simulated user update failure");
        assert_eq!(steps(&log), vec!["delete", "user_authority", "update"]);
    }
}
