use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use async_trait::async_trait;
use oxidauth_kernel::{
    JsonValue,
    auth::{
        authenticate::{AuthenticateParams, AuthenticateServiceTrait},
        authenticate_or_register::*,
        register::{RegisterParams, RegisterServiceTrait},
    },
    authorities::{AuthorityNotFoundError, find_authority_by_client_key::FindAuthorityByClientKey},
    error::BoxedError,
    user_authorities::UserAuthorityNotFoundError,
    users::UserKind,
};
use oxidauth_repository::{
    auth::tree::PermissionTreeQuery,
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery,
    refresh_tokens::insert_refresh_token::InsertRefreshTokenQuery,
    totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery,
    user_authorities::{
        insert_user_authority::InsertUserAuthorityQuery,
        select_user_authority_by_authority_id_and_user_identifier::SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery,
    },
    users::{insert_user::InsertUserQuery, select_user_by_id_query::SelectUserByIdQuery},
};
use serde::Serialize;

use super::{
    authenticate::AuthenticateUseCase,
    register::RegisterUseCase,
    strategies::oauth2::{AuthorityParams, OAuthFlavors},
};
use crate::auth::strategies::oauth2::{
    google::{exchange_google_token, retrieve_google_profile},
    microsoft::{exchange_microsoft_token, retrieve_microsoft_profile},
    registrar::Oauth2RegisterParams,
};

pub struct AuthenticateOrRegisterUseCase<A, M, P, R, S, T, UI, U, UU>
where
    A: InsertUserAuthorityQuery,
    M: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    R: InsertRefreshTokenQuery,
    S: SelectTOTPSecrețByUserIdQuery,
    T: SelectAuthorityByClientKeyQuery,
    UI: InsertUserQuery,
    U: SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery,
    UU: SelectUserByIdQuery,
{
    authenticate: Arc<AuthenticateUseCase<T, U, P, M, R, S, UU>>,
    register: Arc<RegisterUseCase<T, UI, A, P, M, R>>,
    authorities: T,
}

impl<A, M, P, R, S, T, UI, U, UU> AuthenticateOrRegisterUseCase<A, M, P, R, S, T, UI, U, UU>
where
    A: InsertUserAuthorityQuery,
    M: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    R: InsertRefreshTokenQuery,
    S: SelectTOTPSecrețByUserIdQuery,
    T: SelectAuthorityByClientKeyQuery,
    UI: InsertUserQuery,
    U: SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery,
    UU: SelectUserByIdQuery,
{
    pub fn new(
        authenticate: Arc<AuthenticateUseCase<T, U, P, M, R, S, UU>>,
        register: Arc<RegisterUseCase<T, UI, A, P, M, R>>,
        authorities: T,
    ) -> Self {
        Self {
            authenticate,
            register,
            authorities,
        }
    }

    async fn fetch_profile(
        &self,
        authority_params: &AuthorityParams,
        authenticate_params: &OAuth2AuthenticateParams,
    ) -> Result<OAuth2Profile, BoxedError> {
        match authority_params.flavor {
            OAuthFlavors::Google => {
                let code = authenticate_params
                    .code
                    .clone();

                let access_token = exchange_google_token(code, authority_params).await?;

                retrieve_google_profile(access_token, authority_params).await
            },
            OAuthFlavors::Microsoft => {
                let access_token =
                    exchange_microsoft_token(&authenticate_params.code, authority_params).await?;

                retrieve_microsoft_profile(access_token, authority_params).await
            },
        }
    }
}

#[async_trait]
impl<A, M, P, R, S, T, UI, U, UU> AuthenticateOrRegisterServiceTrait
    for AuthenticateOrRegisterUseCase<A, M, P, R, S, T, UI, U, UU>
where
    A: InsertUserAuthorityQuery,
    M: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    R: InsertRefreshTokenQuery,
    S: SelectTOTPSecrețByUserIdQuery,
    T: SelectAuthorityByClientKeyQuery,
    UI: InsertUserQuery,
    U: SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery,
    UU: SelectUserByIdQuery,
{
    #[tracing::instrument(
        name = "AuthenticateOrRegisterUseCase::authenticate_or_register",
        skip(self)
    )]
    async fn authenticate_or_register(
        &self,
        params: &AuthenticateOrRegisterParams,
    ) -> Result<AuthenticateOrRegisterResponse, BoxedError> {
        let Some(authority) = self
            .authorities
            .select_authority_by_client_key(&FindAuthorityByClientKey {
                client_key: params.client_key,
            })
            .await?
        else {
            return Err(AuthorityNotFoundError::client_key(params.client_key).into());
        };

        // VERIFY THE STATE HASH VALUE
        let Ok(parsed_state_hash) = PasswordHash::new(&params.state) else {
            return Err("could not hash state".to_string().into());
        };

        let is_state_hash_verified = Argon2::default()
            .verify_password(
                authority
                    .client_key
                    .as_bytes(),
                &parsed_state_hash,
            )
            .is_ok();

        if !is_state_hash_verified {
            return Err("Invalid state hash".to_string().into());
        }

        // GET PARAMS
        let authority_params: AuthorityParams = authority.params.try_into()?;
        let authenticate_params: OAuth2AuthenticateParams = params
            .params
            .clone()
            .try_into()?;

        let profile = self
            .fetch_profile(&authority_params, &authenticate_params)
            .await?;

        #[derive(Debug, Serialize)]
        struct AuthParams {
            email: String,
        }

        let auth_params = serde_json::to_value(AuthParams {
            email: profile.email.clone(),
        })?;

        let authenticated = self
            .authenticate
            .authenticate(&AuthenticateParams {
                client_key: params.client_key,
                params: JsonValue::new(auth_params),
            })
            .await;

        match authenticated {
            Ok(auth) => {
                return Ok(AuthenticateOrRegisterResponse {
                    jwt: auth.jwt,
                    refresh_token: auth.refresh_token,
                    client_base: authority_params.client_base_url,
                    email: profile.email.clone(),
                    given_name: profile.given_name.clone(),
                    family_name: profile.family_name.clone(),
                    user_id: auth.user_id,
                });
            },
            Err(err) => {
                if err
                    .downcast_ref::<UserAuthorityNotFoundError>()
                    .is_some()
                {
                    let reg_params = Oauth2RegisterParams {
                        first_name: profile.given_name.clone(),
                        last_name: profile.family_name.clone(),
                        email: Some(profile.email.clone()),
                        username: profile.email.clone(),
                        kind: Some(UserKind::Human),
                    };

                    let reg_params_json = serde_json::to_value(reg_params)?;

                    let result = self
                        .register
                        .register(&RegisterParams {
                            client_key: params.client_key,
                            params: JsonValue::new(reg_params_json),
                        })
                        .await?;

                    let res = AuthenticateOrRegisterResponse {
                        jwt: result.jwt,
                        refresh_token: result.refresh_token,
                        client_base: authority_params.client_base_url,
                        email: profile.email.clone(),
                        given_name: profile.given_name,
                        family_name: profile.family_name,
                        user_id: result.user_id,
                    };

                    return Ok(res);
                }

                return Err(err);
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::{Arc, LazyLock, Mutex},
        time::Duration,
    };

    use argon2::{PasswordHasher, password_hash::SaltString};
    use chrono::Utc;
    use oxidauth_kernel::{
        JsonValue,
        auth::tree::{PermissionTree, PermissionsResponse, UserNode},
        authorities::{
            Authority,
            AuthoritySettings,
            AuthorityStatus,
            AuthorityStrategy,
            NbfOffset,
            TotpSettings,
            find_authority_by_client_key::FindAuthorityByClientKey,
        },
        jwt::{EntitlementsEncoding, Jwt},
        private_keys::{PrivateKey, find_most_recent_private_key::FindMostRecentPrivateKey},
        refresh_tokens::{RefreshToken, create_refresh_token::CreateRefreshToken},
        rsa::KeyPair,
        totp_secrets::{TOTPSecret, find_totp_secret_by_user_id::FindTOTPSecretByUserId},
        user_authorities::UserAuthority,
        users::{
            User,
            UserKind,
            UserStatus,
            create_user::CreateUser,
            find_user_by_id::FindUserById,
        },
    };
    use oxidauth_repository::{
        auth::tree::PermissionSearch,
        user_authorities::{
            insert_user_authority::InsertUserAuthority,
            select_user_authority_by_authority_id_and_user_identifier::SelectUserAuthorityByAuthorityIdAndUserIdentifierQueryParams,
        },
    };
    use serde_json::{Value, json};
    use url::Url;
    use uuid::Uuid;

    use super::*;

    const CLIENT_KEY: Uuid = uuid::uuid!("4d1b2c3d-4444-4000-8000-000000000001");
    const AUTHORITY_ID: Uuid = uuid::uuid!("4d1b2c3d-4444-4000-8000-000000000002");
    const EXISTING_USER_ID: Uuid = uuid::uuid!("4d1b2c3d-4444-4000-8000-000000000003");
    const NEW_USER_ID: Uuid = uuid::uuid!("4d1b2c3d-4444-4000-8000-000000000004");
    const NEW_TOKEN_ID: Uuid = uuid::uuid!("4d1b2c3d-4444-4000-8000-000000000005");
    const JWT_TTL_SECS: u64 = 900;
    const REFRESH_TTL_SECS: u64 = 86_400;
    const PROFILE_EMAIL: &str = "ann@example.com";
    const CLIENT_BASE: &str = "https://app.example.com/";

    static KEYS: LazyLock<(String, Vec<u8>)> = LazyLock::new(|| {
        let pair = KeyPair::new().expect("keypair");
        let b64 = pair.base64_encode();
        (
            String::from_utf8(b64.private.clone()).expect("utf8 b64"),
            pair.public.clone(),
        )
    });

    // the state param must be an argon2 PHC hash of the authority's client_key
    // bytes; hashing is deterministic with a fixed salt
    fn state_hash(secret: &[u8]) -> String {
        let salt = SaltString::new("TtHtIsSaLtVaLuE9").expect("salt");
        Argon2::default()
            .hash_password(secret, &salt)
            .expect("hash")
            .to_string()
    }

    // ------------------------------------------------------------------
    // fake identity provider: a loopback HTTP server serving the token
    // exchange and profile endpoints from the authority params. one shared
    // response body satisfies both google and microsoft wire formats
    // ------------------------------------------------------------------
    const TOKEN_BODY: &str = r#"{"access_token":"access-token-123","expires_in":3599,"scope":"openid email profile","token_type":"Bearer","id_token":"idtok","ext_expires_in":3599}"#;
    const PROFILE_BODY: &str = r#"{"email":"ann@example.com","verified_email":true,"given_name":"Ann","family_name":"Last","mail":"ann@example.com","givenName":"Ann","surname":"Last","id":"ms-object-1"}"#;

    struct FakeIdp {
        base: String,
        requests: Arc<Mutex<Vec<String>>>,
    }

    impl FakeIdp {
        fn requests(&self) -> Vec<String> {
            self.requests
                .lock()
                .expect("idp log")
                .clone()
        }

        fn params_json(&self, flavor: &str) -> Value {
            json!({
                "exchange_url": format!("{}/token", self.base),
                "flavor": flavor,
                "oauth2_id": "sso-client-id",
                "oauth2_secret": "sso-client-secret",
                "profile_url": format!("{}/me", self.base),
                "scopes": "openid email profile",
                "redirect_url": "https://authorize.example.com/oauth",
                "client_base_url": CLIENT_BASE,
                "redirect_uri": "https://app.example.com/cb",
            })
        }
    }

    fn start_fake_idp() -> FakeIdp {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind idp");
        let addr = listener
            .local_addr()
            .expect("idp addr");
        let requests: Arc<Mutex<Vec<String>>> = Arc::default();
        let log = requests.clone();

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else {
                    continue;
                };
                let mut buf: Vec<u8> = Vec::new();
                let mut chunk = [0u8; 2048];
                loop {
                    match stream.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            buf.extend_from_slice(&chunk[..n]);
                            if buf
                                .windows(4)
                                .any(|w| w == b"\r\n\r\n")
                            {
                                break;
                            }
                        },
                    }
                }
                let text = String::from_utf8_lossy(&buf).to_string();
                // drain the form body so the client is not blocked on write
                if let Some(cl) = text
                    .lines()
                    .find_map(|l| l.strip_prefix("content-length:"))
                {
                    let want: usize = cl.trim().parse().unwrap_or(0);
                    let have = buf.len().saturating_sub(
                        text.find("\r\n\r\n")
                            .map(|i| i + 4)
                            .unwrap_or(buf.len()),
                    );
                    let mut left = want.saturating_sub(have);
                    while left > 0 {
                        match stream.read(&mut chunk) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => left -= n.min(left),
                        }
                    }
                }
                let first_line = text
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_owned();
                let mut parts = first_line.split_whitespace();
                let method = parts
                    .next()
                    .unwrap_or("")
                    .to_owned();
                let path = parts
                    .next()
                    .unwrap_or("")
                    .to_owned();
                log.lock()
                    .expect("idp log")
                    .push(format!("{method} {path}"));

                let body = if path.ends_with("/me") {
                    PROFILE_BODY
                } else {
                    TOKEN_BODY
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: \
                     {}\r\nconnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        FakeIdp {
            base: format!("http://{addr}"),
            requests,
        }
    }

    // ------------------------------------------------------------------
    // mocks
    // ------------------------------------------------------------------
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

    #[derive(Clone)]
    enum AuthorityMode {
        Missing,
        Oauth2 {
            flavor: &'static str,
            idp_base: String,
        },
        GarbageParams,
    }

    struct MockAuthority {
        mode: AuthorityMode,
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

            match &self.mode {
                AuthorityMode::Missing => Ok(None),
                AuthorityMode::Oauth2 { flavor, idp_base } => {
                    Ok(Some(authority(json!({
                        "exchange_url": format!("{idp_base}/token"),
                        "flavor": flavor,
                        "oauth2_id": "sso-client-id",
                        "oauth2_secret": "sso-client-secret",
                        "profile_url": format!("{idp_base}/me"),
                        "scopes": "openid email profile",
                        "redirect_url": "https://authorize.example.com/oauth",
                        "client_base_url": CLIENT_BASE,
                        "redirect_uri": "https://app.example.com/cb",
                    }))))
                },
                AuthorityMode::GarbageParams => Ok(Some(authority(json!({ "garbage": true })))),
            }
        }
    }

    fn authority(params: Value) -> Authority {
        Authority {
            id: AUTHORITY_ID,
            name: "oauth2-authority".to_owned(),
            client_key: CLIENT_KEY,
            status: AuthorityStatus::Enabled,
            strategy: AuthorityStrategy::Oauth2,
            settings: AuthoritySettings {
                jwt_ttl: Duration::from_secs(JWT_TTL_SECS),
                jwt_nbf_offset: NbfOffset::Disabled,
                refresh_token_ttl: Duration::from_secs(REFRESH_TTL_SECS),
                totp: TotpSettings::Disabled,
                entitlements_encoding: EntitlementsEncoding::Txt,
            },
            params: JsonValue::new(params),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Clone, Copy)]
    enum LookupOutcome {
        Found,
        NotFound,
        Fail,
    }

    struct MockUserAuthority {
        outcome: LookupOutcome,
        log: CallLog,
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
                // the typed error the postgres repo returns when the row is
                // missing; the AOR use case downcasts it to decide to register
                LookupOutcome::NotFound => {
                    Err(UserAuthorityNotFoundError::new(
                        req.authority_id,
                        req.user_identifier.clone(),
                    ))
                },
                LookupOutcome::Found => {
                    Ok(UserAuthority {
                        user_id: EXISTING_USER_ID,
                        authority_id: req.authority_id,
                        user_identifier: req.user_identifier.clone(),
                        params: JsonValue::empty(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    })
                },
            }
        }
    }

    struct MockPermissionTree {
        log: CallLog,
    }

    #[async_trait]
    impl PermissionTreeQuery for MockPermissionTree {
        async fn permission_tree(
            &self,
            params: &PermissionSearch,
        ) -> Result<PermissionsResponse, BoxedError> {
            self.log
                .push("permission_tree".to_owned());

            Ok(PermissionsResponse {
                tree: PermissionTree::User(UserNode {
                    user: test_user(EXISTING_USER_ID),
                    roles: vec![],
                    permissions: vec![],
                }),
                permissions: vec!["oxidauth:**:**".to_owned()],
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

    // totp is disabled in these fixtures: consulting the secret repo fails loud
    struct MockTotpSecret {
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
            Err("totp must not be consulted while disabled".into())
        }
    }

    // OXA-000009: the authenticate use case now loads the user on every path
    // (the status gate), so this mock answers with an enabled user and logs
    // the consultation instead of booby-trapping it.
    struct MockUserById {
        log: CallLog,
    }

    #[async_trait]
    impl SelectUserByIdQuery for MockUserById {
        async fn select_user_by_id(&self, req: &FindUserById) -> Result<User, BoxedError> {
            self.log
                .push(format!("find_user:{}", req.user_id));
            Ok(test_user(req.user_id))
        }
    }

    struct MockInsertUser {
        log: CallLog,
    }

    #[async_trait]
    impl InsertUserQuery for MockInsertUser {
        async fn insert_user(&self, req: &CreateUser) -> Result<User, BoxedError> {
            self.log.push(format!(
                "insert_user:{}.{}",
                req.username,
                serde_json::to_value(
                    req.kind
                        .clone()
                        .unwrap_or_default()
                )
                .expect("kind json")
                .as_str()
                .expect("kind string")
            ));

            Ok(User {
                id: NEW_USER_ID,
                kind: req
                    .kind
                    .clone()
                    .unwrap_or_default(),
                status: req
                    .status
                    .clone()
                    .unwrap_or_default(),
                username: req.username.clone(),
                email: req.email.clone(),
                first_name: req.first_name.clone(),
                last_name: req.last_name.clone(),
                profile: json!({}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    struct MockInsertUserAuthority {
        log: CallLog,
    }

    #[async_trait]
    impl InsertUserAuthorityQuery for MockInsertUserAuthority {
        async fn call(
            &self,
            params: impl Into<InsertUserAuthority> + Send + std::fmt::Debug + 'async_trait,
        ) -> Result<UserAuthority, BoxedError> {
            let params: InsertUserAuthority = params.into();

            self.log.push(format!(
                "insert_user_authority:{}.{}.{}",
                params.user_id, params.authority_id, params.user_identifier
            ));

            Ok(UserAuthority {
                user_id: params.user_id,
                authority_id: params.authority_id,
                user_identifier: params.user_identifier,
                params: JsonValue::empty(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    fn test_user(user_id: Uuid) -> User {
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

    type AU = AuthenticateUseCase<
        MockAuthority,
        MockUserAuthority,
        MockPermissionTree,
        MockPrivateKeys,
        MockRefreshTokens,
        MockTotpSecret,
        MockUserById,
    >;
    type RU = RegisterUseCase<
        MockAuthority,
        MockInsertUser,
        MockInsertUserAuthority,
        MockPermissionTree,
        MockPrivateKeys,
        MockRefreshTokens,
    >;
    type AOR = AuthenticateOrRegisterUseCase<
        MockInsertUserAuthority,
        MockPrivateKeys,
        MockPermissionTree,
        MockRefreshTokens,
        MockTotpSecret,
        MockAuthority,
        MockInsertUser,
        MockUserAuthority,
        MockUserById,
    >;

    fn use_case(log: &CallLog, mode: AuthorityMode, lookup: LookupOutcome) -> AOR {
        let authenticate = Arc::new(AU::new(
            MockAuthority {
                mode: mode.clone(),
                log: log.clone(),
            },
            MockUserAuthority {
                outcome: lookup,
                log: log.clone(),
            },
            MockPermissionTree { log: log.clone() },
            MockPrivateKeys { log: log.clone() },
            MockRefreshTokens { log: log.clone() },
            MockTotpSecret { log: log.clone() },
            MockUserById { log: log.clone() },
        ));
        let register = Arc::new(RU::new(
            MockAuthority {
                mode: mode.clone(),
                log: log.clone(),
            },
            MockInsertUser { log: log.clone() },
            MockInsertUserAuthority { log: log.clone() },
            MockPermissionTree { log: log.clone() },
            MockPrivateKeys { log: log.clone() },
            MockRefreshTokens { log: log.clone() },
        ));

        AuthenticateOrRegisterUseCase::new(
            authenticate,
            register,
            MockAuthority {
                mode,
                log: log.clone(),
            },
        )
    }

    fn oauth2_mode(flavor: &'static str, idp_base: String) -> AuthorityMode {
        AuthorityMode::Oauth2 { flavor, idp_base }
    }

    fn req(state: String) -> AuthenticateOrRegisterParams {
        AuthenticateOrRegisterParams {
            client_key: CLIENT_KEY,
            state,
            params: JsonValue::new(json!({
                "code": "one-time-code",
                "scope": null,
                "client_key": CLIENT_KEY,
            })),
        }
    }

    fn valid_state() -> String {
        state_hash(CLIENT_KEY.as_bytes())
    }

    #[tokio::test]
    async fn missing_authority_short_circuits_with_no_further_calls() {
        let log = CallLog::default();
        let case = use_case(&log, AuthorityMode::Missing, LookupOutcome::Found);

        let err = case
            .authenticate_or_register(&req(valid_state()))
            .await
            .expect_err("a missing authority must error");

        assert!(
            err.to_string()
                .contains("authority not found by client_key"),
            "got: {err}"
        );
        assert_eq!(
            log.ops(),
            vec![format!("find_authority:{CLIENT_KEY}")],
            "nothing but the authority lookup may run"
        );
    }

    #[tokio::test]
    async fn unparseable_state_is_rejected_before_any_token_exchange() {
        let log = CallLog::default();
        let idp = start_fake_idp();
        let case = use_case(
            &log,
            oauth2_mode("Google", idp.base.clone()),
            LookupOutcome::Found,
        );

        let err = case
            .authenticate_or_register(&req("not-a-phc-string".to_owned()))
            .await
            .expect_err("an unparseable state must error");

        assert!(
            err.to_string()
                .contains("could not hash state"),
            "got: {err}"
        );
        assert!(idp.requests().is_empty(), "the idp must not be contacted");
    }

    #[tokio::test]
    async fn state_hashing_a_different_secret_is_rejected() {
        let log = CallLog::default();
        let idp = start_fake_idp();
        let case = use_case(
            &log,
            oauth2_mode("Google", idp.base.clone()),
            LookupOutcome::Found,
        );

        let err = case
            .authenticate_or_register(&req(state_hash(b"not-the-client-key")))
            .await
            .expect_err("a state hash of the wrong secret must error");

        assert!(
            err.to_string()
                .contains("Invalid state hash"),
            "got: {err}"
        );
        assert!(idp.requests().is_empty(), "the idp must not be contacted");
    }

    #[tokio::test]
    async fn non_oauth2_authority_params_fail_after_state_verification() {
        let log = CallLog::default();
        let case = use_case(&log, AuthorityMode::GarbageParams, LookupOutcome::Found);

        let err = case
            .authenticate_or_register(&req(valid_state()))
            .await
            .expect_err("params that are not oauth2 authority params must error");

        assert!(
            err.to_string()
                .contains("missing field `exchange_url`"),
            "expected the serde missing-field error, got: {err}"
        );
    }

    #[tokio::test]
    async fn exchange_failure_propagates_before_any_database_work() {
        let log = CallLog::default();
        // bind then drop: the ephemeral port is guaranteed closed
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let dead_port = listener
            .local_addr()
            .expect("addr");
        drop(listener);
        let case = use_case(
            &log,
            oauth2_mode("Google", format!("http://{dead_port}")),
            LookupOutcome::Found,
        );

        let err = case
            .authenticate_or_register(&req(valid_state()))
            .await
            .expect_err("a refused token exchange must error");

        // reqwest 0.12 wraps connect failures in this display line; the
        // platform-specific os error text only appears in the error source
        assert!(
            err.to_string()
                .contains("error sending request"),
            "expected the reqwest transport wrapper, got: {err}"
        );
        assert!(
            !log.has("find_user_authority"),
            "the authenticate step must never start: got {:?}",
            log.ops()
        );
    }

    #[tokio::test]
    async fn existing_user_authority_dispatches_to_authenticate_and_skips_registration() {
        let log = CallLog::default();
        let idp = start_fake_idp();
        let case = use_case(
            &log,
            oauth2_mode("Google", idp.base.clone()),
            LookupOutcome::Found,
        );

        let res = case
            .authenticate_or_register(&req(valid_state()))
            .await
            .expect("an existing user authority must authenticate");

        assert_eq!(res.user_id, EXISTING_USER_ID);
        assert_eq!(res.email, PROFILE_EMAIL);
        assert_eq!(res.given_name.as_deref(), Some("Ann"));
        assert_eq!(res.family_name.as_deref(), Some("Last"));
        assert_eq!(res.client_base, Url::parse(CLIENT_BASE).expect("url"));
        assert_eq!(res.refresh_token, NEW_TOKEN_ID);
        assert_eq!(
            res.jwt.split('.').count(),
            3,
            "the jwt must be a compact jws"
        );

        let claims = Jwt::decode(&res.jwt, &KEYS.1).expect("the jwt must verify");
        assert_eq!(claims.sub, Some(EXISTING_USER_ID));
        assert_eq!(claims.iss.as_deref(), Some("oxidauth"));

        assert_eq!(
            idp.requests(),
            vec!["POST /token".to_owned(), "GET /me".to_owned()],
            "the google flavor exchanges the code, then fetches the profile"
        );

        let ops = log.ops();
        assert_eq!(
            ops.iter()
                .map(|entry| {
                    entry
                        .split(':')
                        .next()
                        .expect("head")
                })
                .collect::<Vec<_>>(),
            vec![
                "find_authority", // AOR's own lookup
                "find_authority", // the authenticate use case's lookup
                "find_user_authority",
                "find_user", // the OXA-000009 status gate
                "private_key",
                "permission_tree",
                "insert_refresh",
            ],
            "the authenticate dispatch must not register: got {ops:?}"
        );
        assert!(
            ops.iter().any(|entry| {
                entry == &format!("find_user_authority:{AUTHORITY_ID}.ann@example.com")
            }),
            "the profile email is the user identifier: got {ops:?}"
        );
        assert!(
            !log.has("insert_user:"),
            "registration must be skipped: got {ops:?}"
        );
    }

    #[tokio::test]
    async fn missing_user_authority_falls_back_to_registration() {
        let log = CallLog::default();
        let idp = start_fake_idp();
        let case = use_case(
            &log,
            oauth2_mode("Google", idp.base.clone()),
            LookupOutcome::NotFound,
        );

        let res = case
            .authenticate_or_register(&req(valid_state()))
            .await
            .expect("a missing user authority must fall through to registration");

        assert_eq!(
            res.user_id, NEW_USER_ID,
            "the newly registered user owns the response"
        );
        assert_eq!(res.email, PROFILE_EMAIL);
        assert_eq!(res.refresh_token, NEW_TOKEN_ID);

        let claims = Jwt::decode(&res.jwt, &KEYS.1).expect("the jwt must verify");
        assert_eq!(claims.sub, Some(NEW_USER_ID));

        let ops = log.ops();
        let heads = ops
            .iter()
            .map(|entry| {
                entry
                    .split(':')
                    .next()
                    .expect("head")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            heads,
            vec![
                "find_authority", // AOR lookup
                "find_authority", // authenticate attempt
                "find_user_authority",
                "find_authority", // register lookup
                "insert_user",
                "insert_user_authority",
                "permission_tree",
                "private_key",
                "insert_refresh",
            ],
            "the not-found error must switch to the full register flow: got {ops:?}"
        );
        assert!(
            ops.iter()
                .any(|entry| entry == &format!("insert_user:{PROFILE_EMAIL}.human")),
            "the registrar keys the user by profile email: got {ops:?}"
        );
        assert!(
            ops.iter().any(|entry| {
                entry
                    == &format!(
                        "insert_user_authority:{NEW_USER_ID}.{AUTHORITY_ID}.{PROFILE_EMAIL}"
                    )
            }),
            "the user authority links new user + authority + email: got {ops:?}"
        );
    }

    #[tokio::test]
    async fn non_notfound_authenticate_errors_propagate_instead_of_registering() {
        let log = CallLog::default();
        let idp = start_fake_idp();
        let case = use_case(
            &log,
            oauth2_mode("Google", idp.base.clone()),
            LookupOutcome::Fail,
        );

        let err = case
            .authenticate_or_register(&req(valid_state()))
            .await
            .expect_err("a plain lookup failure must propagate");

        assert!(
            err.to_string()
                .contains("simulated lookup failure"),
            "got: {err}"
        );
        assert!(
            !log.has("insert_user:"),
            "only the not-found marker may trigger registration: got {:?}",
            log.ops()
        );
    }

    #[tokio::test]
    async fn microsoft_flavor_reads_the_camel_case_profile_fields() {
        let log = CallLog::default();
        let idp = start_fake_idp();
        let case = use_case(
            &log,
            oauth2_mode("Microsoft", idp.base.clone()),
            LookupOutcome::Found,
        );

        let res = case
            .authenticate_or_register(&req(valid_state()))
            .await
            .expect("the microsoft flavor must complete");

        assert_eq!(
            idp.requests(),
            vec!["POST /token".to_owned(), "GET /me".to_owned()],
            "microsoft exchanges at the same exchange_url"
        );
        assert_eq!(
            res.email, PROFILE_EMAIL,
            "email maps from the camelCase mail field"
        );
        assert_eq!(
            res.given_name.as_deref(),
            Some("Ann"),
            "given_name maps directly"
        );
        assert_eq!(
            res.family_name.as_deref(),
            Some("Last"),
            "family_name maps from surname"
        );
        assert_eq!(res.user_id, EXISTING_USER_ID);
    }
}
