use async_trait::async_trait;
use base64::{Engine, prelude::BASE64_STANDARD};
use boringauth::oath::TOTPBuilder;
use chrono::DateTime;
use oxidauth_kernel::{
    auth::tree::PermissionSearch,
    authorities::{
        AuthorityNotFoundError,
        NbfOffset,
        TotpSettings,
        find_authority_by_client_key::FindAuthorityByClientKey,
    },
    error::BoxedError,
    jwt::{DurationDirection, Jwt, epoch_from_now},
    private_keys::find_most_recent_private_key::FindMostRecentPrivateKey,
    refresh_tokens::create_refresh_token::CreateRefreshToken,
    totp::{
        TOTPValidationRes,
        validate::{ValidateTOTP, ValidateTOTPServiceTrait},
    },
    totp_secrets::{TOTPSecret, find_totp_secret_by_user_id::FindTOTPSecretByUserId},
    users::{UserStatus, find_user_by_id::FindUserById},
};
use oxidauth_repository::{
    auth::tree::PermissionTreeQuery,
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery,
    refresh_tokens::insert_refresh_token::InsertRefreshTokenQuery,
    totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery,
    users::select_user_by_id_query::SelectUserByIdQuery,
};

use crate::dev_prelude::epoch;

pub struct ValidateTOTPUseCase<T, K, P, A, R, UU>
where
    T: SelectTOTPSecrețByUserIdQuery,
    K: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    A: SelectAuthorityByClientKeyQuery,
    R: InsertRefreshTokenQuery,
    UU: SelectUserByIdQuery,
{
    secret: T,
    private_keys: K,
    permission_tree: P,
    authority_by_client_key: A,
    refresh_tokens: R,
    user_by_id: UU,
}

impl<T, K, P, A, R, UU> ValidateTOTPUseCase<T, K, P, A, R, UU>
where
    T: SelectTOTPSecrețByUserIdQuery,
    K: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    A: SelectAuthorityByClientKeyQuery,
    R: InsertRefreshTokenQuery,
    UU: SelectUserByIdQuery,
{
    pub fn new(
        secret: T,
        private_keys: K,
        permission_tree: P,
        authority_by_client_key: A,
        refresh_tokens: R,
        user_by_id: UU,
    ) -> Self {
        Self {
            secret,
            private_keys,
            permission_tree,
            authority_by_client_key,
            refresh_tokens,
            user_by_id,
        }
    }
}

#[async_trait]
impl<T, K, P, A, R, UU> ValidateTOTPServiceTrait for ValidateTOTPUseCase<T, K, P, A, R, UU>
where
    T: SelectTOTPSecrețByUserIdQuery,
    K: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    A: SelectAuthorityByClientKeyQuery,
    R: InsertRefreshTokenQuery,
    UU: SelectUserByIdQuery,
{
    #[tracing::instrument(name = "ValidateTOTPUseCase::validate_totp", skip(self))]
    async fn validate_totp(&self, req: &ValidateTOTP) -> Result<TOTPValidationRes, BoxedError> {
        let user_id = req.user_id;

        let authority = self
            .authority_by_client_key
            .select_authority_by_client_key(&FindAuthorityByClientKey {
                client_key: req.client_key,
            })
            .await?
            .ok_or_else(|| AuthorityNotFoundError::client_key(req.client_key))?;

        let totp_ttl = match authority.settings.totp {
            TotpSettings::Enabled { totp_ttl, .. } => totp_ttl.as_secs() as u32,
            TotpSettings::Disabled => {
                return Err("totp_ttl missing because totp is disabled".into());
            },
        };

        // prepare TOTP secret params
        let secret_params = FindTOTPSecretByUserId { user_id };

        // get the secret key for the user by id
        let secret_by_user_id: TOTPSecret = self
            .secret
            .select_totp_secret_by_user_id(&secret_params)
            .await?;

        let now = epoch()?;

        // Code semantics (retires BUGS_AND_NOTES N-5/N-7 claims): boringauth 0.9.0 `is_valid` is
        // stateless — the code is NOT consumed. Default zero tolerance (no ±1 step) means the code
        // is valid only for the remainder of the current `totp_ttl` bucket and is replayable any
        // number of times inside it; a failed post-check step (JWT mint, refresh insert) can be
        // retried with the same code. Burn-on-success replay protection would need
        // last-accepted-step storage here AND in update_password (period 600 reset codes) —
        // see OXA-000055.
        let valid = TOTPBuilder::new()
            .ascii_key(&secret_by_user_id.secret)
            .period(totp_ttl)
            .timestamp(now)
            .finalize()
            .unwrap()
            .is_valid(&req.code);

        if !valid {
            return Err("invalid totp code".into());
        }

        // OXA-000009 (policy A): this is the leg that mints the privileged
        // full-entitlement JWT after 2FA, so it must not honour a disabled
        // user. Like `authenticate`, the gate runs AFTER the credential check
        // (a wrong code still answers "invalid totp code" — no status oracle)
        // and before any jwt material (private key, permission tree, refresh
        // insert) is consulted.
        let user = self
            .user_by_id
            .select_user_by_id(&FindUserById { user_id })
            .await?;

        if matches!(user.status, UserStatus::Disabled) {
            tracing::warn!(user_id = %user.id, "totp validation refused: account is disabled");
            return Err("account is disabled".into());
        }

        // BUILD JWT ----------------------------------------

        let private_key = self
            .private_keys
            .select_most_recent_private_key(&FindMostRecentPrivateKey {})
            .await?;

        let private_key = BASE64_STANDARD.decode(private_key.private_key)?;

        let permissions = self
            .permission_tree
            .permission_tree(&PermissionSearch::User(user_id))
            .await?
            .permissions;

        let mut jwt_builder = Jwt::builder()
            .with_expires_in(authority.settings.jwt_ttl)
            .with_entitlements(
                authority
                    .settings
                    .entitlements_encoding,
                &permissions,
            )
            .with_subject(user_id)
            .with_issuer("oxidauth".to_owned());

        if let NbfOffset::Enabled(value) = authority
            .settings
            .jwt_nbf_offset
        {
            jwt_builder = jwt_builder.with_not_before_from(value);
        };

        let refresh_token_exp_at = epoch_from_now(
            DurationDirection::Add,
            authority
                .settings
                .refresh_token_ttl,
        )
        .map_err(|err| format!("unable to calculate refresh_token_exp_at: {:?}", err))?;

        let refresh_token_exp_at = DateTime::from_timestamp(refresh_token_exp_at as i64, 0)
            .ok_or("unable to convert refresh_token_exp_at to DateTime")?;

        let refresh_token = self
            .refresh_tokens
            .insert_refresh_token(&CreateRefreshToken {
                user_id,
                authority_id: authority.id,
                expires_at: refresh_token_exp_at,
            })
            .await?;

        let jwt = jwt_builder
            .build()
            .map_err(|err| format!("unable to build jwt: {:?}", err))?
            .encode(&private_key)
            .map_err(|err| format!("unable to encode jwt: {:?}", err))?;

        let response = TOTPValidationRes {
            jwt,
            refresh_token: refresh_token.id,
        };

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, LazyLock, Mutex},
        time::Duration,
    };

    use chrono::Utc;
    use oxidauth_kernel::{
        JsonValue,
        auth::tree::{PermissionTree, PermissionsResponse, UserNode},
        authorities::{
            Authority,
            AuthoritySettings,
            AuthorityStatus,
            AuthorityStrategy,
            TotpSettings,
        },
        jwt::{EntitlementsEncoding, Jwt},
        private_keys::PrivateKey,
        refresh_tokens::RefreshToken,
        rsa::KeyPair,
        totp_secrets::TOTPSecret,
        users::{User, UserKind, UserStatus},
    };
    use serde_json::json;
    use url::Url;
    use uuid::Uuid;

    use super::*;

    const SECRET: &str = "12345678901234567890";
    const TOTP_TTL_SECS: u64 = 600;
    const JWT_TTL_SECS: u64 = 900;
    const REFRESH_TTL_SECS: u64 = 86_400;

    static KEYS: LazyLock<(String, Vec<u8>)> = LazyLock::new(|| {
        let pair = KeyPair::new().expect("keypair");
        let b64 = pair.base64_encode();
        (
            String::from_utf8(b64.private.clone()).expect("utf8 b64"),
            pair.public.clone(),
        )
    });

    const CLIENT_KEY: Uuid = uuid::uuid!("1a1b2c3d-1111-4000-8000-000000000001");
    const AUTHORITY_ID: Uuid = uuid::uuid!("1a1b2c3d-1111-4000-8000-000000000002");
    const USER_ID: Uuid = uuid::uuid!("1a1b2c3d-1111-4000-8000-000000000003");
    const NEW_TOKEN_ID: Uuid = uuid::uuid!("1a1b2c3d-1111-4000-8000-000000000004");

    #[derive(Clone, Default)]
    struct CallLog {
        ops: Arc<Mutex<Vec<String>>>,
    }

    impl CallLog {
        fn push(&self, op: impl Into<String>) {
            self.ops
                .lock()
                .expect("log")
                .push(op.into());
        }

        fn ops(&self) -> Vec<String> {
            self.ops
                .lock()
                .expect("log")
                .clone()
        }
    }

    struct MockAuthority {
        state: AuthorityState,
        log: CallLog,
    }

    enum AuthorityState {
        TotpEnabled,
        TotpDisabled,
        Missing,
        Fail,
    }

    #[async_trait]
    impl SelectAuthorityByClientKeyQuery for MockAuthority {
        async fn select_authority_by_client_key(
            &self,
            req: &FindAuthorityByClientKey,
        ) -> Result<Option<Authority>, BoxedError> {
            self.log
                .push(format!("find_authority:{}", req.client_key));

            match self.state {
                AuthorityState::Fail => Err("simulated authority failure".into()),
                AuthorityState::Missing => Ok(None),
                AuthorityState::TotpDisabled => Ok(Some(authority(false))),
                AuthorityState::TotpEnabled => Ok(Some(authority(true))),
            }
        }
    }

    struct MockSecret {
        fail: bool,
        log: CallLog,
    }

    #[async_trait]
    impl SelectTOTPSecrețByUserIdQuery for MockSecret {
        async fn select_totp_secret_by_user_id(
            &self,
            req: &FindTOTPSecretByUserId,
        ) -> Result<TOTPSecret, BoxedError> {
            self.log
                .push(format!("find_secret:{}", req.user_id));

            if self.fail {
                return Err("simulated secret failure".into());
            }

            Ok(TOTPSecret {
                secret: SECRET.to_owned(),
            })
        }
    }

    struct MockPrivateKeys {
        log: CallLog,
    }

    #[async_trait]
    impl SelectMostRecentPrivateKeyQuery for MockPrivateKeys {
        async fn select_most_recent_private_key(
            &self,
            params: &FindMostRecentPrivateKey,
        ) -> Result<PrivateKey, BoxedError> {
            self.log
                .push("private_key".to_owned());

            Ok(PrivateKey {
                id: Uuid::new_v4(),
                private_key: KEYS.0.clone().into_bytes(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    struct MockPermissionTree {
        log: CallLog,
    }

    #[async_trait]
    impl PermissionTreeQuery for MockPermissionTree {
        async fn permission_tree(
            &self,
            req: &PermissionSearch,
        ) -> Result<PermissionsResponse, BoxedError> {
            self.log
                .push("permission_tree".to_owned());

            let user_id = match req {
                PermissionSearch::User(user_id) => *user_id,
                PermissionSearch::Role(role_id) => *role_id,
            };

            Ok(PermissionsResponse {
                tree: PermissionTree::User(UserNode {
                    user: user(user_id),
                    roles: vec![],
                    permissions: vec![],
                }),
                permissions: vec!["oxidauth:**:**".to_owned()],
            })
        }
    }

    struct MockRefreshTokens {
        log: CallLog,
    }

    #[async_trait]
    impl InsertRefreshTokenQuery for MockRefreshTokens {
        async fn insert_refresh_token(
            &self,
            req: &CreateRefreshToken,
        ) -> Result<RefreshToken, BoxedError> {
            self.log.push(format!(
                "insert_refresh:{}.{}.{}",
                req.user_id,
                req.authority_id,
                req.expires_at.timestamp()
            ));

            Ok(RefreshToken {
                id: NEW_TOKEN_ID,
                user_id: req.user_id,
                authority_id: req.authority_id,
                expires_at: req.expires_at,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    struct MockUserById {
        status: UserStatus,
        fail: bool,
        log: CallLog,
    }

    #[async_trait]
    impl SelectUserByIdQuery for MockUserById {
        async fn select_user_by_id(&self, req: &FindUserById) -> Result<User, BoxedError> {
            self.log
                .push(format!("find_user:{}", req.user_id));

            if self.fail {
                return Err("simulated user lookup failure".into());
            }

            Ok(User {
                status: self.status.clone(),
                ..user(req.user_id)
            })
        }
    }

    fn authority(totp_enabled: bool) -> Authority {
        Authority {
            id: AUTHORITY_ID,
            name: "totp-authority".to_owned(),
            client_key: CLIENT_KEY,
            status: AuthorityStatus::Enabled,
            strategy: AuthorityStrategy::UsernamePassword,
            settings: AuthoritySettings {
                jwt_ttl: Duration::from_secs(JWT_TTL_SECS),
                jwt_nbf_offset: NbfOffset::Disabled,
                refresh_token_ttl: Duration::from_secs(REFRESH_TTL_SECS),
                totp: if totp_enabled {
                    TotpSettings::Enabled {
                        totp_ttl: Duration::from_secs(TOTP_TTL_SECS),
                        webhook: Url::parse("https://totp-relay.invalid/send").expect("url"),
                        webhook_key: "relay-key".to_owned(),
                    }
                } else {
                    TotpSettings::Disabled
                },
                entitlements_encoding: EntitlementsEncoding::Txt,
            },
            params: JsonValue::new(json!({})),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn user(user_id: Uuid) -> User {
        User {
            id: user_id,
            kind: UserKind::Human,
            status: UserStatus::Enabled,
            username: "totp-user".to_owned(),
            email: None,
            first_name: None,
            last_name: None,
            profile: json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn use_case(
        log: &CallLog,
        state: AuthorityState,
        secret_fails: bool,
    ) -> ValidateTOTPUseCase<
        MockSecret,
        MockPrivateKeys,
        MockPermissionTree,
        MockAuthority,
        MockRefreshTokens,
        MockUserById,
    > {
        use_case_with_status(log, state, secret_fails, UserStatus::Enabled)
    }

    fn use_case_with_status(
        log: &CallLog,
        state: AuthorityState,
        secret_fails: bool,
        user_status: UserStatus,
    ) -> ValidateTOTPUseCase<
        MockSecret,
        MockPrivateKeys,
        MockPermissionTree,
        MockAuthority,
        MockRefreshTokens,
        MockUserById,
    > {
        ValidateTOTPUseCase::new(
            MockSecret {
                fail: secret_fails,
                log: log.clone(),
            },
            MockPrivateKeys { log: log.clone() },
            MockPermissionTree { log: log.clone() },
            MockAuthority {
                state,
                log: log.clone(),
            },
            MockRefreshTokens { log: log.clone() },
            MockUserById {
                status: user_status,
                fail: false,
                log: log.clone(),
            },
        )
    }

    fn totp_code(secret: &str) -> String {
        // mirrors the use-case builder: ascii key, authority totp_ttl period,
        // timestamp pinned to the current epoch second
        TOTPBuilder::new()
            .ascii_key(secret)
            .period(TOTP_TTL_SECS as u32)
            .timestamp(epoch().expect("epoch"))
            .finalize()
            .expect("totp config is valid")
            .generate()
    }

    async fn stay_inside_totp_window() {
        // `is_valid` runs with zero tolerance: a code is only valid inside its
        // current bucket; near a boundary, generation and validation can
        // straddle two counters, so wait for the next bucket to start.
        while epoch()
            .expect("epoch")
            .rem_euclid(TOTP_TTL_SECS as i64)
            > TOTP_TTL_SECS as i64 - 5
        {
            tokio::time::sleep(Duration::from_millis(1_100)).await;
        }
    }

    #[tokio::test]
    async fn wrong_code_rejects_before_fetching_signing_material() {
        // read-order pin: the code is checked immediately after the secret is
        // fetched, so a bad code must never reach the private key, the
        // permission tree, or the refresh-token insert
        stay_inside_totp_window().await;

        let valid = totp_code(SECRET);
        let wrong = if valid == "000000" {
            "111111"
        } else {
            "000000"
        };

        let log = CallLog::default();
        let case = use_case(&log, AuthorityState::TotpEnabled, false);

        let err = case
            .validate_totp(&ValidateTOTP {
                user_id: USER_ID,
                code: wrong.to_owned(),
                client_key: CLIENT_KEY,
            })
            .await
            .expect_err("an invalid code must error");

        assert_eq!(err.to_string(), "invalid totp code");
        assert_eq!(
            log.ops(),
            vec![
                format!("find_authority:{CLIENT_KEY}"),
                format!("find_secret:{USER_ID}"),
            ],
            "no jwt material may be fetched after a rejected code"
        );
    }

    #[tokio::test]
    async fn valid_code_for_a_disabled_user_mints_nothing() {
        // OXA-000009 (policy A): the post-2FA full-entitlement mint must not
        // honour a disabled account. The gate sits after the code check (see
        // the sibling test below) and before every jwt-material fetch.
        stay_inside_totp_window().await;

        let code = totp_code(SECRET);
        let log = CallLog::default();
        let case = use_case_with_status(
            &log,
            AuthorityState::TotpEnabled,
            false,
            UserStatus::Disabled,
        );

        let err = case
            .validate_totp(&ValidateTOTP {
                user_id: USER_ID,
                code,
                client_key: CLIENT_KEY,
            })
            .await
            .expect_err("a disabled account must not complete 2FA");

        assert_eq!(err.to_string(), "account is disabled");
        assert_eq!(
            log.ops(),
            vec![
                format!("find_authority:{CLIENT_KEY}"),
                format!("find_secret:{USER_ID}"),
                format!("find_user:{USER_ID}"),
            ],
            "pinned order: the status gate runs after the code check and before \
             the private key, permission tree, and refresh insert"
        );
    }

    #[tokio::test]
    async fn wrong_code_for_a_disabled_user_still_reports_the_code_error() {
        // No status oracle: without a valid code, a disabled account answers
        // exactly like everyone else — the code error — and the user row is
        // never consulted.
        stay_inside_totp_window().await;

        let valid = totp_code(SECRET);
        let wrong = if valid == "000000" {
            "111111"
        } else {
            "000000"
        };

        let log = CallLog::default();
        let case = use_case_with_status(
            &log,
            AuthorityState::TotpEnabled,
            false,
            UserStatus::Disabled,
        );

        let err = case
            .validate_totp(&ValidateTOTP {
                user_id: USER_ID,
                code: wrong.to_owned(),
                client_key: CLIENT_KEY,
            })
            .await
            .expect_err("an invalid code must error");

        assert_eq!(err.to_string(), "invalid totp code");
        assert_eq!(
            log.ops(),
            vec![
                format!("find_authority:{CLIENT_KEY}"),
                format!("find_secret:{USER_ID}"),
            ],
            "the status gate must not run for a rejected code"
        );
    }

    #[tokio::test]
    async fn valid_code_issues_jwt_claims_and_an_inserted_refresh_token() {
        stay_inside_totp_window().await;

        let code = totp_code(SECRET);
        let log = CallLog::default();
        let case = use_case(&log, AuthorityState::TotpEnabled, false);

        let res = case
            .validate_totp(&ValidateTOTP {
                user_id: USER_ID,
                code,
                client_key: CLIENT_KEY,
            })
            .await
            .expect("a valid code must validate");

        // the private key is stored base64-encoded and must be decoded before
        // signing: a successful decode against the pair's public key proves it
        let claims = Jwt::decode(&res.jwt, &KEYS.1).expect("jwt verifies with the keypair");
        assert_eq!(claims.sub, Some(USER_ID));
        assert_eq!(claims.iss.as_deref(), Some("oxidauth"));
        assert_eq!(claims.exp - claims.iat.expect("iat"), JWT_TTL_SECS as usize);
        assert_eq!(
            claims
                .entitlements
                .expect("entitlements")
                .as_vec()
                .expect("txt"),
            vec!["oxidauth:**:**".to_owned()]
        );

        assert_eq!(
            res.refresh_token, NEW_TOKEN_ID,
            "the response carries the inserted token's id"
        );

        let insert = log
            .ops()
            .into_iter()
            .find(|op| op.starts_with("insert_refresh:"))
            .expect("a refresh token must be inserted on success");
        let parts: Vec<_> = insert.split('.').collect();
        assert_eq!(parts[0], format!("insert_refresh:{USER_ID}"));
        assert_eq!(parts[1], AUTHORITY_ID.to_string());
        let exp_at: i64 = parts[2]
            .parse()
            .expect("epoch");
        let now = Utc::now().timestamp();
        assert!(
            (exp_at - (now + REFRESH_TTL_SECS as i64)).abs() <= 2,
            "the new token expires one authority refresh_ttl from now, got {exp_at}"
        );
    }

    #[tokio::test]
    async fn authority_with_totp_disabled_rejects_without_a_ttl() {
        let log = CallLog::default();
        let case = use_case(&log, AuthorityState::TotpDisabled, false);

        let err = case
            .validate_totp(&ValidateTOTP {
                user_id: USER_ID,
                code: "123456".to_owned(),
                client_key: CLIENT_KEY,
            })
            .await
            .expect_err("a disabled totp authority cannot validate codes");

        assert_eq!(err.to_string(), "totp_ttl missing because totp is disabled");
        assert_eq!(
            log.ops(),
            vec![format!("find_authority:{CLIENT_KEY}")],
            "the secret must not even be fetched when totp is disabled"
        );
    }

    #[tokio::test]
    async fn unknown_client_key_is_reported_as_authority_not_found() {
        let log = CallLog::default();
        let case = use_case(&log, AuthorityState::Missing, false);

        let err = case
            .validate_totp(&ValidateTOTP {
                user_id: USER_ID,
                code: "123456".to_owned(),
                client_key: CLIENT_KEY,
            })
            .await
            .expect_err("an unknown authority must error");

        assert!(
            err.to_string()
                .contains(&format!("authority not found by client_key: {CLIENT_KEY}")),
            "got {err}"
        );
        assert_eq!(log.ops().len(), 1);
    }

    #[tokio::test]
    async fn authority_lookup_failure_propagates() {
        let log = CallLog::default();
        let case = use_case(&log, AuthorityState::Fail, false);

        let err = case
            .validate_totp(&ValidateTOTP {
                user_id: USER_ID,
                code: "123456".to_owned(),
                client_key: CLIENT_KEY,
            })
            .await
            .expect_err("repository failures must propagate");

        assert!(
            err.to_string()
                .contains("simulated authority failure")
        );
    }

    #[tokio::test]
    async fn secret_lookup_failure_propagates_before_any_jwt_material() {
        let log = CallLog::default();
        let case = use_case(&log, AuthorityState::TotpEnabled, true);

        let err = case
            .validate_totp(&ValidateTOTP {
                user_id: USER_ID,
                code: "123456".to_owned(),
                client_key: CLIENT_KEY,
            })
            .await
            .expect_err("a secret lookup failure must propagate");

        assert!(
            err.to_string()
                .contains("simulated secret failure")
        );
        assert_eq!(
            log.ops(),
            vec![
                format!("find_authority:{CLIENT_KEY}"),
                format!("find_secret:{USER_ID}"),
            ]
        );
    }
}
