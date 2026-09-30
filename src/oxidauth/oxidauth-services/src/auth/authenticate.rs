use async_trait::async_trait;
use base64::{Engine, prelude::BASE64_STANDARD};
use boringauth::oath::TOTPBuilder;
use chrono::DateTime;
pub use oxidauth_kernel::{
    auth::{
        Authenticator,
        authenticate::{
            AuthenticateParams,
            AuthenticateResponse,
            AuthenticateServiceTrait,
            WebhookReq,
            WebhookRes,
        },
    },
    authorities::{Authority, AuthorityNotFoundError, AuthorityStrategy, TotpSettings},
    error::BoxedError,
    jwt::{Jwt, epoch_from_now},
    private_keys::find_most_recent_private_key::FindMostRecentPrivateKey,
    totp_secrets::{TOTPSecret, find_totp_secret_by_user_id::FindTOTPSecretByUserId},
    users::find_user_by_id::FindUserById,
};
use oxidauth_kernel::{authorities::NbfOffset, jwt::DurationDirection, users::UserStatus};
use oxidauth_repository::{
    auth::tree::{PermissionSearch, PermissionTreeQuery},
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery,
    refresh_tokens::insert_refresh_token::{CreateRefreshToken, InsertRefreshTokenQuery},
    totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery,
    user_authorities::select_user_authority_by_authority_id_and_user_identifier::{
        SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery,
        SelectUserAuthorityByAuthorityIdAndUserIdentifierQueryParams,
    },
    users::select_user_by_id_query::SelectUserByIdQuery,
};
use reqwest::Client;
use tracing::info;

use crate::{auth::strategies::*, bootstrap::TOTP_VALIDATE_PERMISSION, dev_prelude::epoch};

pub struct AuthenticateUseCase<T, U, P, M, R, S, UU>
where
    T: SelectAuthorityByClientKeyQuery,
    U: SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
    S: SelectTOTPSecrețByUserIdQuery,
    UU: SelectUserByIdQuery,
{
    authority_by_client_key: T,
    user_authority: U,
    permission_tree: P,
    private_keys: M,
    refresh_tokens: R,
    user_totp_secret: S,
    user_by_id: UU,
}

impl<T, U, P, M, R, S, UU> AuthenticateUseCase<T, U, P, M, R, S, UU>
where
    T: SelectAuthorityByClientKeyQuery,
    U: SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
    S: SelectTOTPSecrețByUserIdQuery,
    UU: SelectUserByIdQuery,
{
    pub fn new(
        authority_by_client_key: T,
        user_authority: U,
        permission_tree: P,
        private_keys: M,
        refresh_tokens: R,
        user_totp_secret: S,
        user_by_id: UU,
    ) -> Self {
        Self {
            authority_by_client_key,
            user_authority,
            permission_tree,
            private_keys,
            refresh_tokens,
            user_totp_secret,
            user_by_id,
        }
    }
}

#[async_trait]
impl<T, U, P, M, R, S, UU> AuthenticateServiceTrait for AuthenticateUseCase<T, U, P, M, R, S, UU>
where
    T: SelectAuthorityByClientKeyQuery,
    U: SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
    S: SelectTOTPSecrețByUserIdQuery,
    UU: SelectUserByIdQuery,
{
    #[tracing::instrument(name = "AuthenticateUseCase::authenticate", skip(self))]
    async fn authenticate(
        &self,
        params: &AuthenticateParams,
    ) -> Result<AuthenticateResponse, BoxedError> {
        let authority = self
            .authority_by_client_key
            .select_authority_by_client_key(&params.into())
            .await?
            .ok_or_else(|| AuthorityNotFoundError::client_key(params.client_key))?;

        let authenticator = build_authenticator(&authority).await?;

        let user_identifier = authenticator
            .user_identifier_from_request(&params.params)
            .await?;

        let user_authority_params = SelectUserAuthorityByAuthorityIdAndUserIdentifierQueryParams {
            authority_id: authority.id,
            user_identifier: user_identifier.clone(),
        };

        let user_authority = self
            .user_authority
            .select_user_authority_by_authority_id_and_user_identifier(&user_authority_params)
            .await?;

        let _ = authenticator
            .authenticate(params.params.clone(), &authority, &user_authority)
            .await?;

        // OXA-000009 (policy A — disable means disable): the user row is loaded
        // unconditionally — it gates status AND feeds the 2FA webhook below —
        // and the gate runs AFTER the authenticator, so a wrong password still
        // reports the password error: `disabled` is not an oracle for callers
        // without valid credentials. Only `Disabled` is refused; `Invited`
        // accounts keep authenticating (OXA-000009 Step 0 decision, see
        // changelog — meaningful `Invited` semantics is a separate product item).
        let user = self
            .user_by_id
            .select_user_by_id(&FindUserById {
                user_id: user_authority.user_id,
            })
            .await?;

        if matches!(user.status, UserStatus::Disabled) {
            tracing::warn!(user_id = %user.id, "login refused: account is disabled");
            return Err("account is disabled".into());
        }

        let private_key = self
            .private_keys
            .select_most_recent_private_key(&FindMostRecentPrivateKey {})
            .await?;

        let private_key = BASE64_STANDARD.decode(private_key.private_key)?;

        let mut jwt_builder = Jwt::builder()
            .with_subject(user_authority.user_id)
            .with_issuer("oxidauth".to_owned());

        if let NbfOffset::Enabled(value) = authority
            .settings
            .jwt_nbf_offset
        {
            jwt_builder = jwt_builder.with_not_before_from(value);
        };

        match authority.settings.totp {
            TotpSettings::Enabled {
                totp_ttl,
                webhook,
                webhook_key,
            } => {
                info!("login requires 2FA");

                jwt_builder = jwt_builder
                    .with_expires_in(totp_ttl)
                    .with_entitlements(
                        authority
                            .settings
                            .entitlements_encoding,
                        &[TOTP_VALIDATE_PERMISSION.to_string()],
                    );

                // get the secret key for the user by id
                let secret_by_user_id: TOTPSecret = self
                    .user_totp_secret
                    .select_totp_secret_by_user_id(&FindTOTPSecretByUserId {
                        user_id: user_authority.user_id,
                    })
                    .await?;

                let now = epoch()?;

                let code = TOTPBuilder::new()
                    .ascii_key(&secret_by_user_id.secret)
                    .period(totp_ttl.as_secs() as u32)
                    .timestamp(now)
                    .finalize()
                    .map_err(|err| format!("error generating totp: {:?}", err))?
                    .generate();

                let name = match (user.first_name, user.last_name) {
                    (None, None) => None,
                    (None, Some(last)) => Some(last),
                    (Some(first), None) => Some(first),
                    (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
                };

                let email = user
                    .email
                    .ok_or("unable to send totp code - missing email")?;

                let webhook_params = WebhookReq {
                    webhook_key,
                    name,
                    email,
                    code,
                };

                let client = Client::new();
                let webhook_res: WebhookRes = client
                    .post(webhook)
                    .json(&webhook_params)
                    .send()
                    .await?
                    .json()
                    .await?;

                if !webhook_res.success {
                    return Err(format!("unable to send totp code to: {}", user_identifier,).into());
                }
            },
            TotpSettings::Disabled => {
                let permissions = self
                    .permission_tree
                    .permission_tree(&PermissionSearch::User(user_authority.user_id))
                    .await?
                    .permissions;

                jwt_builder = jwt_builder
                    .with_expires_in(authority.settings.jwt_ttl)
                    .with_entitlements(
                        authority
                            .settings
                            .entitlements_encoding,
                        &permissions,
                    );
            },
        }

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
                user_id: user_authority.user_id,
                authority_id: authority.id,
                expires_at: refresh_token_exp_at,
            })
            .await?;

        let jwt = jwt_builder
            .build()
            .map_err(|err| format!("unable to build jwt: {:?}", err))?
            .encode(&private_key)
            .map_err(|err| format!("unable to encode jwt: {:?}", err))?;

        let response = AuthenticateResponse {
            jwt,
            refresh_token: refresh_token.id,
            user_id: user_authority.user_id,
        };

        Ok(response)
    }
}

/// build_authenticator hydrates and returns an Authenticator
/// It takes the Authority (which has settings/params) and the strategy
/// and returns a Box<dyn Authenticator>
pub async fn build_authenticator(
    authority: &Authority,
) -> Result<Box<dyn Authenticator>, BoxedError> {
    use AuthorityStrategy::*;

    match authority.strategy {
        UsernamePassword => username_password::authenticator::new(authority).await,
        SingleUseToken => unimplemented!(),
        Oauth2 => oauth2::authenticator::new(authority).await,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::TcpListener,
        sync::{Arc, LazyLock, Mutex},
        time::Duration,
    };

    use chrono::Utc;
    use oxidauth_kernel::{
        JsonValue,
        auth::tree::{PermissionTree, PermissionsResponse, UserNode},
        authorities::find_authority_by_client_key::FindAuthorityByClientKey,
        private_keys::PrivateKey,
        refresh_tokens::RefreshToken,
        rsa::KeyPair,
        user_authorities::{UserAuthority, UserAuthorityNotFoundError},
        users::{User, UserKind, UserStatus},
    };
    use serde_json::{Value, json};
    use url::Url;
    use uuid::Uuid;

    use super::*;
    use crate::{
        EnvGuard,
        auth::strategies::username_password::fixtures::{
            AUTHORITY_ID,
            CLIENT_KEY,
            PASSWORD_PEPPER,
            PASSWORD_SALT,
            USER_ID,
            authority_params_json,
            stored_password_hash,
        },
    };

    const PASSWORD: &str = "correct-password";
    const TOTP_SECRET: &str = "12345678901234567890";
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

        fn has(&self, op: &str) -> bool {
            self.ops()
                .iter()
                .any(|entry| entry.starts_with(op))
        }
    }

    enum AuthorityState {
        Found,
        TotpEnabled { webhook: Url },
        Missing,
        Fail,
    }

    struct MockAuthority {
        state: AuthorityState,
        log: CallLog,
    }

    #[async_trait]
    impl SelectAuthorityByClientKeyQuery for MockAuthority {
        async fn select_authority_by_client_key(
            &self,
            req: &FindAuthorityByClientKey,
        ) -> Result<Option<Authority>, BoxedError> {
            self.log
                .push(format!("find_authority:{}", req.client_key));

            match &self.state {
                AuthorityState::Fail => Err("simulated authority failure".into()),
                AuthorityState::Missing => Ok(None),
                AuthorityState::Found => Ok(Some(fixtures_authority(TotpSettings::Disabled))),
                AuthorityState::TotpEnabled { webhook } => {
                    Ok(Some(fixtures_authority(TotpSettings::Enabled {
                        totp_ttl: Duration::from_secs(TOTP_TTL_SECS),
                        webhook: webhook.clone(),
                        webhook_key: "relay-key".to_owned(),
                    })))
                },
            }
        }
    }

    fn fixtures_authority(totp: TotpSettings) -> Authority {
        let mut authority = crate::auth::strategies::username_password::fixtures::authority(
            authority_params_json(),
        );
        authority.settings.totp = totp;
        authority
            .settings
            .jwt_nbf_offset = NbfOffset::Enabled(Duration::from_secs(30));
        authority
    }

    struct MockUserAuthority {
        outcome: LookupOutcome,
        log: CallLog,
    }

    enum LookupOutcome {
        Found,
        NotFound,
        Fail,
    }

    #[async_trait]
    impl SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery for MockUserAuthority {
        async fn select_user_authority_by_authority_id_and_user_identifier(
            &self,
            req: &SelectUserAuthorityByAuthorityIdAndUserIdentifierQueryParams,
        ) -> Result<UserAuthority, BoxedError> {
            self.log.push(format!(
                "find_user_authority:{}.{}",
                req.authority_id, req.user_identifier
            ));

            match self.outcome {
                LookupOutcome::Fail => Err("simulated lookup failure".into()),
                LookupOutcome::NotFound => {
                    Err(UserAuthorityNotFoundError::new(
                        req.authority_id,
                        req.user_identifier.clone(),
                    ))
                },
                LookupOutcome::Found => {
                    Ok(UserAuthority {
                        user_id: USER_ID,
                        authority_id: req.authority_id,
                        user_identifier: req.user_identifier.clone(),
                        params: JsonValue::new(json!({
                            "password_hash": stored_password_hash(PASSWORD, PASSWORD_SALT, PASSWORD_PEPPER),
                        })),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    })
                },
            }
        }
    }

    struct MockPermissionTree {
        fail: bool,
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

            if self.fail {
                return Err("simulated permission tree failure".into());
            }

            let user_id = match req {
                PermissionSearch::User(user_id) => *user_id,
                PermissionSearch::Role(role_id) => *role_id,
            };

            Ok(PermissionsResponse {
                tree: PermissionTree::User(UserNode {
                    user: test_user(user_id, true),
                    roles: vec![],
                    permissions: vec![],
                }),
                permissions: vec!["oxidauth:**:**".to_owned(), "app:read:all".to_owned()],
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

    struct MockTotpSecret {
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
                .push(format!("find_secret:{}", req.user_id));

            if self.fail {
                return Err("simulated secret failure".into());
            }

            Ok(TOTPSecret {
                secret: TOTP_SECRET.to_owned(),
            })
        }
    }

    struct MockUserById {
        email: Option<&'static str>,
        fail: bool,
        status: UserStatus,
        log: CallLog,
    }

    #[async_trait]
    impl SelectUserByIdQuery for MockUserById {
        async fn select_user_by_id(&self, req: &FindUserById) -> Result<User, BoxedError> {
            self.log
                .push(format!("find_user:{}", req.user_id));

            if self.fail {
                return Err("simulated user failure".into());
            }

            Ok(User {
                id: req.user_id,
                kind: UserKind::Human,
                status: self.status.clone(),
                username: "test-user".to_owned(),
                email: self.email.map(str::to_owned),
                first_name: Some("Ann".to_owned()),
                last_name: Some("Last".to_owned()),
                profile: json!({}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    const NEW_TOKEN_ID: Uuid = uuid::uuid!("2b1b2c3d-2222-4000-8000-000000000005");

    fn test_user(user_id: Uuid, _with_email: bool) -> User {
        User {
            id: user_id,
            kind: UserKind::Human,
            status: UserStatus::Enabled,
            username: "test-user".to_owned(),
            email: None,
            first_name: None,
            last_name: None,
            profile: json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Default)]
    struct Wiring {
        authority: Option<AuthorityState>,
        lookup: Option<LookupOutcome>,
        tree_fails: bool,
        secret_fails: bool,
        user_email: Option<&'static str>,
        user_fails: bool,
        user_status: UserStatus,
    }

    fn use_case(
        log: &CallLog,
        wiring: Wiring,
    ) -> AuthenticateUseCase<
        MockAuthority,
        MockUserAuthority,
        MockPermissionTree,
        MockPrivateKeys,
        MockRefreshTokens,
        MockTotpSecret,
        MockUserById,
    > {
        AuthenticateUseCase::new(
            MockAuthority {
                state: wiring
                    .authority
                    .unwrap_or(AuthorityState::Found),
                log: log.clone(),
            },
            MockUserAuthority {
                outcome: wiring
                    .lookup
                    .unwrap_or(LookupOutcome::Found),
                log: log.clone(),
            },
            MockPermissionTree {
                fail: wiring.tree_fails,
                log: log.clone(),
            },
            MockPrivateKeys { log: log.clone() },
            MockRefreshTokens { log: log.clone() },
            MockTotpSecret {
                fail: wiring.secret_fails,
                log: log.clone(),
            },
            MockUserById {
                email: wiring.user_email,
                fail: wiring.user_fails,
                status: wiring.user_status,
                log: log.clone(),
            },
        )
    }

    fn login_params(password: &str) -> AuthenticateParams {
        AuthenticateParams {
            client_key: CLIENT_KEY,
            params: JsonValue::new(json!({
                "username": "test-user",
                "password": password,
            })),
        }
    }

    fn totp_code(secret: &str) -> String {
        TOTPBuilder::new()
            .ascii_key(secret)
            .period(TOTP_TTL_SECS as u32)
            .timestamp(epoch().expect("epoch"))
            .finalize()
            .expect("totp config is valid")
            .generate()
    }

    async fn stay_inside_totp_window() {
        while epoch()
            .expect("epoch")
            .rem_euclid(TOTP_TTL_SECS as i64)
            > TOTP_TTL_SECS as i64 - 5
        {
            tokio::time::sleep(Duration::from_millis(1_100)).await;
        }
    }

    /// One-shot devserver answering every request with `response_body`,
    /// capturing each request body. Stands in for the customer webhook.
    fn spawn_webhook(response_body: &'static str) -> (Url, Arc<Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind webhook");
        let addr = listener
            .local_addr()
            .expect("addr");
        let bodies: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let thread_bodies = bodies.clone();

        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut writer = stream
                    .try_clone()
                    .expect("clone stream");
                let mut reader = BufReader::new(stream);
                let mut content_length = 0usize;

                loop {
                    let mut line = String::new();
                    if reader
                        .read_line(&mut line)
                        .unwrap_or(0)
                        == 0
                        || line.trim().is_empty()
                    {
                        break;
                    }
                    if line
                        .to_ascii_lowercase()
                        .starts_with("content-length:")
                    {
                        content_length = line[15..]
                            .trim()
                            .parse()
                            .unwrap_or(0);
                    }
                }

                let mut body = vec![0u8; content_length];
                if reader
                    .read_exact(&mut body)
                    .is_err()
                {
                    break;
                }
                thread_bodies
                    .lock()
                    .expect("bodies")
                    .push(String::from_utf8_lossy(&body).to_string());

                let header = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: \
                     {}\r\nconnection: close\r\n\r\n",
                    response_body.len()
                );
                if writer
                    .write_all(header.as_bytes())
                    .and_then(|_| writer.write_all(response_body.as_bytes()))
                    .is_err()
                {
                    break;
                }
                let _ = writer.flush();
            }
        });

        (
            Url::parse(&format!("http://{addr}/send")).expect("webhook url"),
            bodies,
        )
    }

    #[tokio::test]
    async fn correct_password_issues_jwt_with_claims_and_a_refresh_token() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let log = CallLog::default();
        let case = use_case(&log, Wiring::default());

        let res = case
            .authenticate(&login_params(PASSWORD))
            .await
            .expect("the stored password must authenticate");

        assert_eq!(res.user_id, USER_ID);
        assert_eq!(res.refresh_token, NEW_TOKEN_ID);

        let claims = Jwt::decode(&res.jwt, &KEYS.1).expect("jwt verifies with the keypair");
        assert_eq!(claims.sub, Some(USER_ID));
        assert_eq!(claims.iss.as_deref(), Some("oxidauth"));
        let iat = claims
            .iat
            .expect("iat is set");
        assert_eq!(claims.exp - iat, JWT_TTL_SECS as usize);
        // the fixture authority carries NbfOffset::Enabled(30s) — the builder's
        // implicit default is 10s, so this pins with_not_before_from(value)
        assert_eq!(
            claims
                .nbf
                .expect("nbf is emitted"),
            iat - 30
        );
        assert_eq!(
            claims
                .entitlements
                .expect("entitlements")
                .as_vec()
                .expect("txt"),
            vec!["oxidauth:**:**".to_owned(), "app:read:all".to_owned()]
        );

        let insert = log
            .ops()
            .into_iter()
            .find(|op| op.starts_with("insert_refresh:"))
            .expect("refresh token inserted");
        let parts: Vec<_> = insert.split('.').collect();
        assert_eq!(parts[0], format!("insert_refresh:{USER_ID}"));
        assert_eq!(parts[1], AUTHORITY_ID.to_string());
        let exp_at: i64 = parts[2]
            .parse()
            .expect("epoch");
        assert!(
            (exp_at - (Utc::now().timestamp() + REFRESH_TTL_SECS as i64)).abs() <= 2,
            "refresh token expires one authority refresh_ttl from now"
        );
        assert!(
            log.has(&format!("find_user_authority:{AUTHORITY_ID}.test-user")),
            "the lookup is keyed by (resolved authority, extracted username)"
        );
        assert!(
            log.has(&format!("find_user:{USER_ID}")),
            "the OXA-000009 status gate consults the resolved user"
        );
    }

    #[tokio::test]
    async fn disabled_user_is_refused_after_the_password_check_with_no_token_material() {
        // OXA-000009 (policy A): a disabled account with the CORRECT password
        // gets nothing — the gate fires before the signing key, the permission
        // tree, and the refresh-token insert.
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let log = CallLog::default();
        let case = use_case(
            &log,
            Wiring {
                user_status: UserStatus::Disabled,
                ..Default::default()
            },
        );

        let err = case
            .authenticate(&login_params(PASSWORD))
            .await
            .expect_err("a disabled account must not authenticate");

        assert_eq!(err.to_string(), "account is disabled");

        let ops = log.ops();
        let gate_at = ops
            .iter()
            .position(|op| op.starts_with("find_user:"))
            .expect("the gate loads the user");
        let lookup_at = ops
            .iter()
            .position(|op| op.starts_with("find_user_authority:"))
            .expect("the user authority resolves first");
        assert!(
            gate_at > lookup_at,
            "pinned order: the gate runs after the credential path, got {ops:?}"
        );
        assert!(
            !log.has("private_key"),
            "no signing key for a disabled account"
        );
        assert!(!log.has("permission_tree"));
        assert!(
            !log.has("insert_refresh"),
            "no jwt and no refresh token may be minted for a disabled account"
        );
    }

    #[tokio::test]
    async fn disabled_user_is_refused_before_the_2fa_relay_runs() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let (webhook, bodies) = spawn_webhook(r#"{"success":true}"#);
        let log = CallLog::default();
        let case = use_case(
            &log,
            Wiring {
                authority: Some(AuthorityState::TotpEnabled { webhook }),
                user_email: Some("ann@example.com"),
                user_status: UserStatus::Disabled,
                ..Default::default()
            },
        );

        let err = case
            .authenticate(&login_params(PASSWORD))
            .await
            .expect_err("a disabled account must not start a 2FA session");

        assert_eq!(err.to_string(), "account is disabled");
        assert!(
            bodies
                .lock()
                .expect("bodies")
                .is_empty(),
            "the relay may not be contacted for a disabled account"
        );
        assert!(
            !log.has("find_secret:"),
            "the gate precedes the whole totp arm — no code is ever sent"
        );
        assert!(!log.has("insert_refresh"));
    }

    #[tokio::test]
    async fn wrong_password_fails_before_any_token_material_is_fetched() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let log = CallLog::default();
        let case = use_case(&log, Wiring::default());

        case.authenticate(&login_params("wrong-password"))
            .await
            .expect_err("a wrong password must not authenticate");

        assert!(log.has("find_user_authority"));
        assert!(
            !log.has("find_user:"),
            "the status gate must not run before the credential check"
        );
        assert!(
            !log.has("private_key"),
            "no signing key after a failed login"
        );
        assert!(!log.has("permission_tree"));
        assert!(!log.has("insert_refresh"));
    }

    #[tokio::test]
    async fn malformed_login_params_reject_before_the_user_authority_lookup() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let log = CallLog::default();
        let case = use_case(&log, Wiring::default());

        case.authenticate(&AuthenticateParams {
            client_key: CLIENT_KEY,
            params: JsonValue::new(json!({ "passcode": 42 })),
        })
        .await
        .expect_err("params without a username must error");

        assert!(!log.has("find_user_authority"));
        assert!(!log.has("insert_refresh"));
    }

    #[tokio::test]
    async fn missing_pepper_env_fails_authenticator_construction() {
        let _unset = EnvGuard::unset("OXIDAUTH_USERNAME_PASSWORD_PEPPER");

        let log = CallLog::default();
        let case = use_case(&log, Wiring::default());

        let err = case
            .authenticate(&login_params(PASSWORD))
            .await
            .expect_err("the username_password strategy requires a pepper");

        assert!(
            format!("{err:?}").contains("OXIDAUTH_USERNAME_PASSWORD_PEPPER")
                || err
                    .to_string()
                    .contains("environment variable not found"),
            "got {err:?}"
        );
        assert!(log.has(&format!("find_authority:{CLIENT_KEY}")));
        assert!(!log.has("find_user_authority"), "fails before the lookup");
    }

    #[tokio::test]
    async fn unknown_client_key_is_reported_as_authority_not_found() {
        let log = CallLog::default();
        let case = use_case(
            &log,
            Wiring {
                authority: Some(AuthorityState::Missing),
                ..Default::default()
            },
        );

        let err = case
            .authenticate(&login_params(PASSWORD))
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
    #[should_panic(expected = "not implemented")]
    async fn single_use_token_strategy_is_not_implemented() {
        let authority = Authority {
            strategy: AuthorityStrategy::SingleUseToken,
            ..fixtures_authority(TotpSettings::Disabled)
        };

        // build_authenticator hits unimplemented!() for this strategy
        let _ = build_authenticator(&authority).await;
    }

    #[tokio::test]
    async fn permission_tree_failure_propagates_without_inserting_a_token() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let log = CallLog::default();
        let case = use_case(
            &log,
            Wiring {
                tree_fails: true,
                ..Default::default()
            },
        );

        let err = case
            .authenticate(&login_params(PASSWORD))
            .await
            .expect_err("tree failures must propagate");

        assert!(
            err.to_string()
                .contains("simulated permission tree failure")
        );
        assert!(!log.has("insert_refresh"));
    }
    #[tokio::test]
    async fn lookup_failures_propagate() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        for (wiring, message) in [
            (
                Wiring {
                    lookup: Some(LookupOutcome::Fail),
                    ..Default::default()
                },
                "simulated lookup failure",
            ),
            (
                Wiring {
                    lookup: Some(LookupOutcome::NotFound),
                    ..Default::default()
                },
                "user authority not found",
            ),
        ] {
            let log = CallLog::default();
            let case = use_case(&log, wiring);

            let err = case
                .authenticate(&login_params(PASSWORD))
                .await
                .expect_err("lookup failures must propagate");

            assert!(
                err.to_string()
                    .contains(message),
                "got {err}"
            );
        }
    }

    #[tokio::test]
    async fn totp_path_secret_and_user_failures_propagate() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        // the relay is never reached in these legs; port 1 makes any leak loud
        let webhook = Url::parse("http://127.0.0.1:1/send").expect("url");

        for (wiring, message) in [
            (
                Wiring {
                    authority: Some(AuthorityState::TotpEnabled {
                        webhook: webhook.clone(),
                    }),
                    user_fails: true,
                    ..Default::default()
                },
                "simulated user failure",
            ),
            (
                Wiring {
                    authority: Some(AuthorityState::TotpEnabled {
                        webhook: webhook.clone(),
                    }),
                    secret_fails: true,
                    user_email: Some("ann@example.com"),
                    ..Default::default()
                },
                "simulated secret failure",
            ),
        ] {
            let log = CallLog::default();
            let case = use_case(&log, wiring);

            let err = case
                .authenticate(&login_params(PASSWORD))
                .await
                .expect_err("2FA-path repository failures must propagate");

            assert!(
                err.to_string()
                    .contains(message),
                "got {err}"
            );
        }
    }

    #[tokio::test]
    async fn totp_enabled_authority_intercepts_with_a_limited_token_and_webhook() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);
        stay_inside_totp_window().await;

        let (webhook, bodies) = spawn_webhook(r#"{"success":true}"#);
        let log = CallLog::default();
        let case = use_case(
            &log,
            Wiring {
                authority: Some(AuthorityState::TotpEnabled {
                    webhook: webhook.clone(),
                }),
                user_email: Some("ann@example.com"),
                ..Default::default()
            },
        );

        let res = case
            .authenticate(&login_params(PASSWORD))
            .await
            .expect("2FA interception must still mint a limited session token");

        let claims = Jwt::decode(&res.jwt, &KEYS.1).expect("jwt verifies");
        assert_eq!(
            claims
                .entitlements
                .expect("entitlements")
                .as_vec()
                .expect("txt"),
            vec![TOTP_VALIDATE_PERMISSION.to_owned()],
            "a pending-2FA token may only call the totp validation endpoint"
        );
        let iat = claims.iat.expect("iat");
        assert_eq!(
            claims.exp - iat,
            TOTP_TTL_SECS as usize,
            "the pending token expires with the authority totp_ttl, not jwt_ttl"
        );
        assert_eq!(
            res.refresh_token, NEW_TOKEN_ID,
            "a refresh token is minted too"
        );

        let bodies = bodies.lock().expect("bodies");
        assert_eq!(bodies.len(), 1, "the webhook is called exactly once");
        let sent: Value = serde_json::from_str(&bodies[0]).expect("webhook json body");
        assert_eq!(sent["webhook_key"], json!("relay-key"));
        assert_eq!(sent["name"], json!("Ann Last"));
        assert_eq!(sent["email"], json!("ann@example.com"));
        assert_eq!(
            sent["code"],
            json!(totp_code(TOTP_SECRET)),
            "the code sent to the relay must be the one derived from the user's secret"
        );
    }

    #[tokio::test]
    async fn totp_interception_without_email_fails_before_the_webhook() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let (webhook, bodies) = spawn_webhook(r#"{"success":true}"#);
        let log = CallLog::default();
        let case = use_case(
            &log,
            Wiring {
                authority: Some(AuthorityState::TotpEnabled {
                    webhook: webhook.clone(),
                }),
                user_email: None,
                ..Default::default()
            },
        );

        let err = case
            .authenticate(&login_params(PASSWORD))
            .await
            .expect_err("there is no way to deliver a code without an email");

        assert!(
            err.to_string()
                .contains("unable to send totp code - missing email"),
            "got {err}"
        );
        assert!(
            bodies
                .lock()
                .expect("bodies")
                .is_empty(),
            "no webhook call"
        );
        assert!(log.has("find_secret"), "the code is still generated first");
    }

    #[tokio::test]
    async fn rejected_webhook_aborts_the_login() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);
        stay_inside_totp_window().await;

        let (webhook, bodies) = spawn_webhook(r#"{"success":false}"#);
        let log = CallLog::default();
        let case = use_case(
            &log,
            Wiring {
                authority: Some(AuthorityState::TotpEnabled {
                    webhook: webhook.clone(),
                }),
                user_email: Some("ann@example.com"),
                ..Default::default()
            },
        );

        let err = case
            .authenticate(&login_params(PASSWORD))
            .await
            .expect_err("a refused relay aborts the login");

        assert!(
            err.to_string()
                .contains("unable to send totp code to: test-user"),
            "got {err}"
        );
        assert_eq!(
            bodies
                .lock()
                .expect("bodies")
                .len(),
            1
        );
        assert!(!log.has("insert_refresh"), "no token on a refused relay");
    }

    #[tokio::test]
    async fn authority_lookup_failures_propagate() {
        let log = CallLog::default();
        let case = use_case(
            &log,
            Wiring {
                authority: Some(AuthorityState::Fail),
                ..Default::default()
            },
        );

        let err = case
            .authenticate(&login_params(PASSWORD))
            .await
            .expect_err("a plain authority failure must propagate");

        assert!(
            err.to_string()
                .contains("simulated authority failure"),
            "got {err}"
        );
    }
}
