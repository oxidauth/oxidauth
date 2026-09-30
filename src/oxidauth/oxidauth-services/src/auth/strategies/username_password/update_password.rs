use async_trait::async_trait;
use boringauth::oath::TOTPBuilder;
use oxidauth_kernel::{
    JsonValue,
    auth::username_password::update_password::{
        UpdatePasswordParams,
        UpdatePasswordResponse,
        UpdatePasswordServiceTrait,
    },
    authorities::find_authority_by_client_key::FindAuthorityByClientKey,
    error::BoxedError,
    totp_secrets::find_totp_secret_by_user_id::FindTOTPSecretByUserId,
    user_authorities::update_user_authority::UpdateUserAuthority,
    users::{UserStatus, find_user_by_id::FindUserById},
};
use oxidauth_repository::{
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery,
    user_authorities::{
        select_user_authority_by_authority_id_and_user_identifier::{
            SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery,
            SelectUserAuthorityByAuthorityIdAndUserIdentifierQueryParams,
        },
        update_user_authority::UpdateUserAuthorityQuery,
    },
    users::select_user_by_id_query::SelectUserByIdQuery,
};

use super::helpers::{hash_password, raw_password_hash};
use crate::{
    auth::strategies::username_password::{AuthorityParams, UserAuthorityParams},
    dev_prelude::epoch,
};

pub struct UpdatePasswordUseCase<S, T, U, V, W>
where
    S: SelectTOTPSecrețByUserIdQuery,
    T: SelectAuthorityByClientKeyQuery,
    U: UpdateUserAuthorityQuery,
    V: SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery,
    W: SelectUserByIdQuery,
{
    user_totp_secret: S,
    authority_by_client_key: T,
    update_user_authority: U,
    select_user_authority: V,
    user_by_id: W,
}

impl<S, T, U, V, W> UpdatePasswordUseCase<S, T, U, V, W>
where
    S: SelectTOTPSecrețByUserIdQuery,
    T: SelectAuthorityByClientKeyQuery,
    U: UpdateUserAuthorityQuery,
    V: SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery,
    W: SelectUserByIdQuery,
{
    pub fn new(
        user_totp_secret: S,
        authority_by_client_key: T,
        update_user_authority: U,
        select_user_authority: V,
        user_by_id: W,
    ) -> Self {
        Self {
            user_totp_secret,
            authority_by_client_key,
            update_user_authority,
            select_user_authority,
            user_by_id,
        }
    }
}

#[async_trait]
impl<S, T, U, V, W> UpdatePasswordServiceTrait for UpdatePasswordUseCase<S, T, U, V, W>
where
    S: SelectTOTPSecrețByUserIdQuery,
    T: SelectAuthorityByClientKeyQuery,
    U: UpdateUserAuthorityQuery,
    V: SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery,
    W: SelectUserByIdQuery,
{
    #[tracing::instrument(name = "UpdatePasswordUseCase::update_password", skip(self))]
    async fn update_password(
        &self,
        params: &UpdatePasswordParams,
    ) -> Result<UpdatePasswordResponse, BoxedError> {
        // check password match
        if params.password != params.password_conf {
            return Err("password and password confirmation do not match".into());
        }

        // get authority params using provided client key
        let authority_res = self
            .authority_by_client_key
            .select_authority_by_client_key(&FindAuthorityByClientKey {
                client_key: params.client_key,
            })
            .await;

        let Ok(Some(authority)) = authority_res else {
            return Err("Failed to find authority by client key".into());
        };

        let user_authority_res = self
            .select_user_authority
            .select_user_authority_by_authority_id_and_user_identifier(
                &SelectUserAuthorityByAuthorityIdAndUserIdentifierQueryParams {
                    authority_id: authority.id,
                    user_identifier: params.username.clone(),
                },
            )
            .await;

        let Ok(user_authority) = user_authority_res else {
            return Err("Failed to find user by username".into());
        };

        // OXA-000009 (policy A — disable means disable): recovery is a credential
        // path, so it fails closed. The user-authority lookup resolves exactly one
        // user (`user_identifier` is UNIQUE), so the status gate can run before
        // the TOTP secret is even fetched: a disabled account may neither
        // spend/confirm a 600 s reset code nor reach the password write. The
        // error is distinct but keeps the existing envelope — the anonymous
        // route still answers 200 {success:false} (typed errors: OXA-000042).
        let user = self
            .user_by_id
            .select_user_by_id(&FindUserById {
                user_id: user_authority.user_id,
            })
            .await?;

        if matches!(user.status, UserStatus::Disabled) {
            tracing::warn!(user_id = %user.id, "password recovery refused: account is disabled");
            return Err("account is disabled".into());
        }

        let secret_by_user_id = self
            .user_totp_secret
            .select_totp_secret_by_user_id(&FindTOTPSecretByUserId {
                user_id: user_authority.user_id,
            })
            .await?;

        let now = epoch()?;

        // Mirror of the code-semantics comment in `totp/validate.rs` (OXA-000055 / OXA-000057):
        // `is_valid` is stateless — zero tolerance, no burn. A reset code stays replayable for the
        // remainder of its 600 s bucket; burn-on-success (OXA-000055 step 3) must land HERE TOO —
        // the 600 s period makes this the widest replay exposure (amplifies OXA-000005).
        let valid = TOTPBuilder::new()
            .ascii_key(&secret_by_user_id.secret)
            .period(600)
            .timestamp(now)
            .finalize()
            .unwrap()
            .is_valid(&params.code);

        if !valid {
            return Err("invalid code".into());
        }

        let authority_params: AuthorityParams = authority.params.try_into()?;

        let password_salt = authority_params.password_salt;

        let password_pepper = std::env::var("OXIDAUTH_USERNAME_PASSWORD_PEPPER")?;

        let password = raw_password_hash(&params.password, &password_salt, &password_pepper);

        let password_hash = hash_password(password).map_err(|err| err.to_string())?;

        let new_password_params = UserAuthorityParams { password_hash };

        let new_password_value = serde_json::to_value(new_password_params)?;

        let user_authority = UpdateUserAuthority {
            user_id: user_authority.user_id,
            authority_id: authority.id,
            params: JsonValue::new(new_password_value),
        };

        let res = self
            .update_user_authority
            .update_user_authority(&user_authority)
            .await;

        Ok(UpdatePasswordResponse {
            success: res.is_ok(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use chrono::Utc;
    use oxidauth_kernel::{
        JsonValue,
        authorities::{
            Authority,
            AuthoritySettings,
            AuthorityStatus,
            AuthorityStrategy,
            TotpSettings,
        },
        jwt::EntitlementsEncoding,
        totp_secrets::TOTPSecret,
        user_authorities::{UserAuthority, UserAuthorityNotFoundError},
        users::{User, UserKind, UserStatus},
    };
    use serde_json::{Value, json};
    use uuid::Uuid;

    use super::*;
    use crate::{
        EnvGuard,
        auth::strategies::username_password::{
            fixtures::{AUTHORITY_ID, CLIENT_KEY, PASSWORD_PEPPER, PASSWORD_SALT, USER_ID},
            helpers::verify_password,
        },
    };

    const TOTP_SECRET: &str = "12345678901234567890";

    #[derive(Clone, Default)]
    struct CallLog {
        authority_requests: Arc<Mutex<Vec<Uuid>>>,
        lookup_requests: Arc<Mutex<Vec<(Uuid, String)>>>,
        user_requests: Arc<Mutex<Vec<Uuid>>>,
        totp_requests: Arc<Mutex<Vec<Uuid>>>,
        update_requests: Arc<Mutex<Vec<(Uuid, Uuid, Value)>>>,
    }

    impl CallLog {
        fn authority_requests(&self) -> Vec<Uuid> {
            self.authority_requests
                .lock()
                .expect("log")
                .clone()
        }

        fn lookup_requests(&self) -> Vec<(Uuid, String)> {
            self.lookup_requests
                .lock()
                .expect("log")
                .clone()
        }

        fn totp_requests(&self) -> Vec<Uuid> {
            self.totp_requests
                .lock()
                .expect("log")
                .clone()
        }

        fn user_requests(&self) -> Vec<Uuid> {
            self.user_requests
                .lock()
                .expect("log")
                .clone()
        }

        fn update_requests(&self) -> Vec<(Uuid, Uuid, Value)> {
            self.update_requests
                .lock()
                .expect("log")
                .clone()
        }
    }

    struct MockAuthorityRepo {
        found: bool,
        log: CallLog,
    }

    #[async_trait]
    impl SelectAuthorityByClientKeyQuery for MockAuthorityRepo {
        async fn select_authority_by_client_key(
            &self,
            req: &FindAuthorityByClientKey,
        ) -> Result<Option<Authority>, BoxedError> {
            self.log
                .authority_requests
                .lock()
                .expect("log")
                .push(req.client_key);

            if !self.found {
                return Ok(None);
            }

            Ok(Some(Authority {
                id: AUTHORITY_ID,
                name: "update-password-authority".to_owned(),
                client_key: CLIENT_KEY,
                status: AuthorityStatus::Enabled,
                strategy: AuthorityStrategy::UsernamePassword,
                settings: AuthoritySettings {
                    jwt_ttl: Duration::from_secs(900),
                    jwt_nbf_offset: Default::default(),
                    refresh_token_ttl: Duration::from_secs(86_400),
                    totp: TotpSettings::Disabled,
                    entitlements_encoding: EntitlementsEncoding::Txt,
                },
                params: JsonValue::new(json!({ "password_salt": PASSWORD_SALT })),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        }
    }

    struct MockUserAuthorityLookup {
        fail: bool,
        log: CallLog,
    }

    #[async_trait]
    impl SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery for MockUserAuthorityLookup {
        async fn select_user_authority_by_authority_id_and_user_identifier(
            &self,
            req: &SelectUserAuthorityByAuthorityIdAndUserIdentifierQueryParams,
        ) -> Result<UserAuthority, BoxedError> {
            self.log
                .lookup_requests
                .lock()
                .expect("log")
                .push((req.authority_id, req.user_identifier.clone()));

            if self.fail {
                return Err(UserAuthorityNotFoundError::new(
                    req.authority_id,
                    req.user_identifier.clone(),
                ));
            }

            Ok(UserAuthority {
                user_id: USER_ID,
                authority_id: req.authority_id,
                user_identifier: req.user_identifier.clone(),
                params: JsonValue::empty(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    struct MockTotpSecret {
        secret: String,
        fail: bool,
        log: CallLog,
    }

    #[async_trait]
    impl SelectTOTPSecrețByUserIdQuery for MockTotpSecret {
        async fn select_totp_secret_by_user_id(
            &self,
            req: &FindTOTPSecretByUserId,
        ) -> Result<TOTPSecret, BoxedError> {
            self.log
                .totp_requests
                .lock()
                .expect("log")
                .push(req.user_id);

            if self.fail {
                return Err("simulated totp secret failure".into());
            }

            Ok(TOTPSecret {
                secret: self.secret.clone(),
            })
        }
    }

    struct MockUpdateUserAuthority {
        fail: bool,
        log: CallLog,
    }

    #[async_trait]
    impl UpdateUserAuthorityQuery for MockUpdateUserAuthority {
        async fn update_user_authority(
            &self,
            req: &UpdateUserAuthority,
        ) -> Result<UserAuthority, BoxedError> {
            self.log
                .update_requests
                .lock()
                .expect("log")
                .push((
                    req.user_id,
                    req.authority_id,
                    req.params
                        .clone()
                        .inner_value(),
                ));

            if self.fail {
                return Err("simulated update failure".into());
            }

            Ok(UserAuthority {
                user_id: req.user_id,
                authority_id: req.authority_id,
                user_identifier: "test-user".to_owned(),
                params: req.params.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    struct MockUserById {
        status: UserStatus,
        log: CallLog,
    }

    #[async_trait]
    impl SelectUserByIdQuery for MockUserById {
        async fn select_user_by_id(&self, req: &FindUserById) -> Result<User, BoxedError> {
            self.log
                .user_requests
                .lock()
                .expect("log")
                .push(req.user_id);

            Ok(User {
                id: req.user_id,
                kind: UserKind::Human,
                status: self.status.clone(),
                username: "test-user".to_owned(),
                email: None,
                first_name: None,
                last_name: None,
                profile: json!({}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    fn use_case(
        log: &CallLog,
        authority_found: bool,
        lookup_fails: bool,
        totp_fails: bool,
        update_fails: bool,
    ) -> UpdatePasswordUseCase<
        MockTotpSecret,
        MockAuthorityRepo,
        MockUpdateUserAuthority,
        MockUserAuthorityLookup,
        MockUserById,
    > {
        use_case_status(
            log,
            authority_found,
            lookup_fails,
            totp_fails,
            update_fails,
            UserStatus::Enabled,
        )
    }

    fn use_case_status(
        log: &CallLog,
        authority_found: bool,
        lookup_fails: bool,
        totp_fails: bool,
        update_fails: bool,
        user_status: UserStatus,
    ) -> UpdatePasswordUseCase<
        MockTotpSecret,
        MockAuthorityRepo,
        MockUpdateUserAuthority,
        MockUserAuthorityLookup,
        MockUserById,
    > {
        UpdatePasswordUseCase::new(
            MockTotpSecret {
                secret: TOTP_SECRET.to_owned(),
                fail: totp_fails,
                log: log.clone(),
            },
            MockAuthorityRepo {
                found: authority_found,
                log: log.clone(),
            },
            MockUpdateUserAuthority {
                fail: update_fails,
                log: log.clone(),
            },
            MockUserAuthorityLookup {
                fail: lookup_fails,
                log: log.clone(),
            },
            MockUserById {
                status: user_status,
                log: log.clone(),
            },
        )
    }

    fn update_params(
        client_key: Uuid,
        code: String,
        password: &str,
        conf: &str,
    ) -> UpdatePasswordParams {
        UpdatePasswordParams {
            code,
            username: "test-user".to_owned(),
            client_key,
            password: password.to_owned(),
            password_conf: conf.to_owned(),
        }
    }

    fn totp_code(secret: &str) -> String {
        // mirrors the use-case builder exactly: ascii key, 600s period,
        // timestamp pinned to the current epoch second
        TOTPBuilder::new()
            .ascii_key(secret)
            .period(600)
            .timestamp(epoch().expect("epoch"))
            .finalize()
            .expect("totp config is valid")
            .generate()
    }

    async fn stay_inside_totp_window() {
        // `is_valid` runs with zero tolerance: a code is only valid inside its
        // 600s bucket. Near a bucket boundary, generation and validation can
        // straddle two counters, so wait for the next bucket to start.
        while epoch()
            .expect("epoch")
            .rem_euclid(600)
            > 595
        {
            tokio::time::sleep(Duration::from_millis(1_100)).await;
        }
    }

    #[tokio::test]
    async fn rejects_confirmation_mismatch_before_touching_any_repository() {
        let log = CallLog::default();
        let case = use_case(&log, true, false, false, false);

        let err = case
            .update_password(&update_params(
                CLIENT_KEY,
                "000000".to_owned(),
                "abc",
                "xyz",
            ))
            .await
            .expect_err("mismatched confirmation must error");

        assert_eq!(
            err.to_string(),
            "password and password confirmation do not match"
        );
        assert!(
            log.authority_requests()
                .is_empty()
        );
        assert!(
            log.lookup_requests()
                .is_empty()
        );
        assert!(log.totp_requests().is_empty());
        assert!(
            log.update_requests()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn unknown_client_key_fails_before_user_lookup() {
        let log = CallLog::default();
        let case = use_case(&log, false, false, false, false);

        let err = case
            .update_password(&update_params(CLIENT_KEY, "000000".to_owned(), "pw", "pw"))
            .await
            .expect_err("unknown client key must error");

        assert!(
            err.to_string()
                .contains("Failed to find authority by client key")
        );
        assert_eq!(
            log.authority_requests()
                .as_slice(),
            &[CLIENT_KEY],
            "lookup must use the requested client key"
        );
        assert!(
            log.lookup_requests()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn user_lookup_failure_is_masked_as_a_username_error() {
        let log = CallLog::default();
        let case = use_case(&log, true, true, false, false);

        let err = case
            .update_password(&update_params(CLIENT_KEY, "000000".to_owned(), "pw", "pw"))
            .await
            .expect_err("lookup failure must error");

        // BUG(pinned): the underlying repository error — the typed
        // UserAuthorityNotFoundError the lookup returns — is discarded; every
        // lookup failure is reported as "Failed to find user by username".
        // Typed-error passthrough is deferred to OXA-000042.
        assert!(
            err.to_string()
                .contains("Failed to find user by username")
        );
        assert!(
            !err.to_string()
                .contains("user authority not found"),
            "pinned: repository error details are swallowed"
        );
        assert_eq!(
            log.lookup_requests()
                .as_slice(),
            &[(AUTHORITY_ID, "test-user".to_owned())],
            "lookup must be keyed by (authority id, username)"
        );
    }

    #[tokio::test]
    async fn rejects_an_invalid_totp_code_without_writing_anything() {
        // Pinned: the audit expected an old-password verification gate, but
        // `UpdatePasswordParams` has no old-password field at all — the only
        // *credential* gate is the TOTP code (the OXA-000009 status gate lives
        // above it, ahead of the secret lookup).
        stay_inside_totp_window().await;

        let valid = totp_code(TOTP_SECRET);
        let wrong = if valid == "000000" {
            "111111"
        } else {
            "000000"
        };

        let log = CallLog::default();
        let case = use_case(&log, true, false, false, false);

        let err = case
            .update_password(&update_params(CLIENT_KEY, wrong.to_owned(), "pw", "pw"))
            .await
            .expect_err("an invalid totp code must error");

        assert_eq!(err.to_string(), "invalid code");
        assert!(
            log.update_requests()
                .is_empty(),
            "an invalid code must not write a new password hash"
        );
        assert_eq!(
            log.totp_requests().as_slice(),
            &[USER_ID],
            "the code must be checked against the resolved user's secret"
        );
    }

    #[tokio::test]
    async fn disabled_user_is_refused_before_the_recovery_code_is_consulted() {
        // OXA-000009 (policy A — disable means disable): recovery is a
        // credential path and fails closed. Even holding a live 600 s code, a
        // disabled account is refused: the gate runs before the secret lookup,
        // so the code is never spent, and the password hash is never rewritten.
        let code = totp_code(TOTP_SECRET);
        let log = CallLog::default();
        let case = use_case_status(&log, true, false, false, false, UserStatus::Disabled);

        let err = case
            .update_password(&update_params(CLIENT_KEY, code, "pw", "pw"))
            .await
            .expect_err("a disabled account must not recover its password");

        assert_eq!(err.to_string(), "account is disabled");
        assert_eq!(
            log.user_requests(),
            &[USER_ID],
            "the gate loads the user behind the resolved user authority"
        );
        assert!(
            log.totp_requests().is_empty(),
            "pinned order: the status gate runs before the totp secret lookup — \
             a disabled user may not even spend/confirm a reset code"
        );
        assert!(
            log.update_requests()
                .is_empty(),
            "the stored password hash must not be overwritten"
        );
    }

    #[tokio::test]
    async fn valid_code_replaces_the_stored_password_hash() {
        stay_inside_totp_window().await;
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let code = totp_code(TOTP_SECRET);
        let log = CallLog::default();
        let case = use_case(&log, true, false, false, false);

        let res = case
            .update_password(&update_params(
                CLIENT_KEY,
                code,
                "brand-new-password",
                "brand-new-password",
            ))
            .await
            .expect("valid code and matching confirmation must update");

        assert!(res.success, "a completed update must report success");
        assert_eq!(
            log.user_requests(),
            &[USER_ID],
            "the status gate consults exactly the resolved user, and an enabled \
             account passes it"
        );

        let updates = log.update_requests();
        assert_eq!(updates.len(), 1, "exactly one user authority update");
        let (user_id, authority_id, params) = &updates[0];
        assert_eq!(*user_id, USER_ID);
        assert_eq!(*authority_id, AUTHORITY_ID);

        let stored = params
            .get("password_hash")
            .and_then(Value::as_str)
            .expect("params carry the new password_hash");
        assert!(stored.starts_with("$argon2"));
        assert_eq!(
            verify_password(
                raw_password_hash("brand-new-password", PASSWORD_SALT, PASSWORD_PEPPER),
                stored.to_owned(),
            ),
            Ok(true),
            "the replaced hash must verify the new salt+pepper material"
        );
    }

    #[tokio::test]
    async fn repository_failure_reports_success_false_instead_of_an_error() {
        stay_inside_totp_window().await;
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let code = totp_code(TOTP_SECRET);
        let log = CallLog::default();
        let case = use_case(&log, true, false, false, true);

        let res = case
            .update_password(&update_params(CLIENT_KEY, code, "pw", "pw"))
            .await
            .expect("BUG(pinned): a repository update failure is swallowed into Ok");

        assert!(
            !res.success,
            "pinned: update failure surfaces as Ok(UpdatePasswordResponse {{ success: false }})"
        );
        assert_eq!(log.update_requests().len(), 1, "the update was attempted");
    }

    #[tokio::test]
    async fn totp_secret_lookup_failure_propagates() {
        let log = CallLog::default();
        let case = use_case(&log, true, false, true, false);

        let err = case
            .update_password(&update_params(CLIENT_KEY, "000000".to_owned(), "pw", "pw"))
            .await
            .expect_err("a totp secret failure must propagate");

        assert!(
            err.to_string()
                .contains("simulated totp secret failure")
        );
        assert!(
            log.update_requests()
                .is_empty()
        );
    }

    #[derive(Clone, Default)]
    struct LogCapture(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for LogCapture {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0
                .lock()
                .expect("log")
                .extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl tracing_subscriber::fmt::MakeWriter<'_> for LogCapture {
        type Writer = Self;

        fn make_writer(&self) -> Self {
            self.clone()
        }
    }

    #[tokio::test]
    async fn tracing_span_logs_never_carry_the_new_password_or_the_recovery_code() {
        // OXA-000008: every `#[tracing::instrument(.., skip(self))]` layer on
        // this use case records the `params` argument through `Debug`, so a
        // derived `Debug` persisted the fresh password (and the TOTP code) in
        // plaintext for every completed reset. The request must still succeed
        // end to end with the masked `Debug`.
        use tracing_subscriber::prelude::*;

        stay_inside_totp_window().await;
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        const SENTINEL_NEW_PW: &str = "SENTINEL-brand-new-password";

        let code = totp_code(TOTP_SECRET);
        let log = CallLog::default();
        let case = use_case(&log, true, false, false, false);
        let params = update_params(CLIENT_KEY, code, SENTINEL_NEW_PW, SENTINEL_NEW_PW);

        let capture = LogCapture::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(capture.clone())
            .with_max_level(tracing::Level::TRACE)
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .finish();
        let _guard = subscriber.set_default();

        let res = case
            .update_password(&params)
            .await
            .expect("valid code and matching confirmation must update");
        drop(_guard);

        assert!(res.success, "the wire path is unchanged by the masking");
        assert_eq!(log.update_requests().len(), 1, "the update ran through");

        let logged = String::from_utf8(
            capture
                .0
                .lock()
                .expect("log")
                .clone(),
        )
        .expect("utf-8 log output");
        assert!(
            !logged.contains(SENTINEL_NEW_PW),
            "the sentinel new password leaked into the tracing log"
        );
        assert!(
            !logged.contains("SENTINEL-"),
            "no sentinel fragment may reach the log"
        );
        assert!(
            logged.contains("password: \"******\"")
                && logged.contains("password_conf: \"******\"")
                && logged.contains("code: \"******\""),
            "the recorded params span field must carry the constant masks, got: {logged}"
        );
    }
}
