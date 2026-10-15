use async_trait::async_trait;
use base64::prelude::*;
use chrono::DateTime;
use oxidauth_kernel::{
    auth::{
        Registrar,
        register::{RegisterParams, RegisterResponse, RegisterServiceTrait},
    },
    authorities::{Authority, AuthorityNotFoundError, AuthorityStrategy, NbfOffset, TotpSettings},
    error::BoxedError,
    jwt::{DurationDirection, Jwt, epoch_from_now},
    private_keys::find_most_recent_private_key::FindMostRecentPrivateKey,
};
use oxidauth_repository::{
    auth::tree::{PermissionSearch, PermissionTreeQuery},
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery,
    refresh_tokens::insert_refresh_token::{CreateRefreshToken, InsertRefreshTokenQuery},
    totp_secrets::insert_totp_secret::{InsertTotpSecretParams, InsertTotpSecretQuery},
    user_authorities::insert_user_authority::InsertUserAuthorityQuery,
    users::insert_user::InsertUserQuery,
};

use crate::{auth::strategies, bootstrap::TOTP_VALIDATE_PERMISSION, random_string};

pub struct RegisterUseCase<T, U, A, P, M, R, S>
where
    T: SelectAuthorityByClientKeyQuery,
    U: InsertUserQuery,
    A: InsertUserAuthorityQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
    S: InsertTotpSecretQuery,
{
    authority_by_client_key: T,
    users: U,
    user_authorities: A,
    permission_tree: P,
    private_keys: M,
    refresh_tokens: R,
    totp_secrets: S,
}

impl<T, U, A, P, M, R, S> RegisterUseCase<T, U, A, P, M, R, S>
where
    T: SelectAuthorityByClientKeyQuery,
    U: InsertUserQuery,
    A: InsertUserAuthorityQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
    S: InsertTotpSecretQuery,
{
    pub fn new(
        authority_by_client_key: T,
        users: U,
        user_authorities: A,
        permission_tree: P,
        private_keys: M,
        refresh_tokens: R,
        totp_secrets: S,
    ) -> Self {
        Self {
            authority_by_client_key,
            users,
            user_authorities,
            permission_tree,
            private_keys,
            refresh_tokens,
            totp_secrets,
        }
    }
}

#[async_trait]
impl<T, U, A, P, M, R, S> RegisterServiceTrait for RegisterUseCase<T, U, A, P, M, R, S>
where
    T: SelectAuthorityByClientKeyQuery,
    U: InsertUserQuery,
    A: InsertUserAuthorityQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
    S: InsertTotpSecretQuery,
{
    #[tracing::instrument(name = "RegisterUseCase::register", skip(self))]
    async fn register(&self, params: &RegisterParams) -> Result<RegisterResponse, BoxedError> {
        let authority = self
            .authority_by_client_key
            .select_authority_by_client_key(&params.into())
            .await?
            .ok_or_else(|| AuthorityNotFoundError::client_key(params.client_key))?;

        let registrar = build_registrar(&authority).await?;

        let (user, user_authority) = registrar
            .register(params.params.clone())
            .await?;

        let user = self
            .users
            .insert_user(&user)
            .await?;

        self.user_authorities
            .call((user.id, &user_authority))
            .await?;

        // Everything the second factor travels with lives in one place: the window
        // the code is valid for and the webhook that carries it.
        let totp = match authority.settings.totp {
            TotpSettings::Enabled {
                totp_ttl,
                webhook,
                webhook_key,
            } => Some((totp_ttl, webhook, webhook_key)),
            TotpSettings::Disabled => None,
        };

        // A new account on a TOTP authority already owes a code, and the code can
        // only come from a secret the account owns: mint one now — the same random
        // string the secret backfill uses — and hand the first code to the customer
        // webhook. If the webhook is down, the user and the secret are already
        // stored, so the login path can email a fresh one: signup stays recoverable
        // instead of stranding an account with no second factor and no way to get
        // one.
        if let Some((totp_ttl, webhook, webhook_key)) = &totp {
            let secret = random_string();

            self.totp_secrets
                .insert_totp_secret(&InsertTotpSecretParams {
                    user_id: user.id,
                    secret_key: secret.clone(),
                })
                .await?;

            crate::totp::email_code(
                webhook.clone(),
                webhook_key.clone(),
                *totp_ttl,
                &secret,
                &user,
                &user_authority.user_identifier,
            )
            .await?;
        }

        // The registration is not a session either: the account arrives owing its
        // first code, so it gets the same limited jwt and the same short-lived,
        // `totp_validated = false` refresh token that the first leg of a login gets.
        // Handing a new account the full tree would make signup a way around the
        // second factor the authority exists to enforce.
        let (permissions, jwt_ttl, token_ttl) = match totp.as_ref().map(|totp| totp.0) {
            Some(totp_ttl) => (
                vec![TOTP_VALIDATE_PERMISSION.to_owned()],
                totp_ttl,
                totp_ttl,
            ),
            None => (
                self.permission_tree
                    .permission_tree(&PermissionSearch::User(user.id))
                    .await?
                    .permissions,
                authority.settings.jwt_ttl,
                authority.settings.refresh_token_ttl,
            ),
        };

        let private_key = self
            .private_keys
            .select_most_recent_private_key(&FindMostRecentPrivateKey {})
            .await?;

        let private_key = BASE64_STANDARD.decode(private_key.private_key)?;

        let mut jwt_builder = Jwt::builder()
            .with_subject(user.id)
            .with_issuer("oxidauth".to_owned())
            .with_expires_in(jwt_ttl)
            .with_entitlements(
                authority
                    .settings
                    .entitlements_encoding,
                &permissions,
            );

        if let NbfOffset::Enabled(value) = authority
            .settings
            .jwt_nbf_offset
        {
            jwt_builder = jwt_builder.with_not_before_from(value);
        };

        let jwt = jwt_builder
            .build()
            .map_err(|err| format!("unable to build jwt: {:?}", err))?
            .encode(&private_key)
            .map_err(|err| format!("unable to encode jwt: {:?}", err))?;

        let refresh_token_exp_at = epoch_from_now(DurationDirection::Add, token_ttl)
            .map_err(|err| format!("unable to calculate refresh_token_exp_at: {:?}", err))?;

        let refresh_token_exp_at = DateTime::from_timestamp(refresh_token_exp_at as i64, 0)
            .ok_or("unable to convert refresh_token_exp_at to DateTime")?;

        let refresh_token = self
            .refresh_tokens
            .insert_refresh_token(&CreateRefreshToken {
                user_id: user.id,
                authority_id: authority.id,
                totp_validated: totp.is_none(),
                expires_at: refresh_token_exp_at,
            })
            .await?;

        Ok(RegisterResponse {
            jwt,
            refresh_token: refresh_token.id,
            user_id: user.id,
        })
    }
}

pub async fn build_registrar(authority: &Authority) -> Result<Box<dyn Registrar>, BoxedError> {
    use AuthorityStrategy::*;

    match authority.strategy {
        UsernamePassword => strategies::username_password::registrar::new(authority).await,
        Oauth2 => strategies::oauth2::registrar::new(authority).await,
        SingleUseToken => unimplemented!(),
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
            AuthoritySettings,
            AuthorityStatus,
            TotpSettings,
            find_authority_by_client_key::FindAuthorityByClientKey,
        },
        jwt::{EntitlementsEncoding, Jwt},
        private_keys::PrivateKey,
        refresh_tokens::RefreshToken,
        rsa::KeyPair,
        totp_secrets::create_totp_secret::CreateTotpSecretResponse,
        user_authorities::UserAuthority,
        users::{
            User,
            UserAlreadyExistsError,
            UserKind,
            UserStatus,
            Username,
            create_user::CreateUser,
        },
    };
    use oxidauth_repository::user_authorities::insert_user_authority::InsertUserAuthority;
    use serde_json::{Value, json};
    use url::Url;
    use uuid::Uuid;

    use super::*;
    use crate::{
        EnvGuard,
        auth::strategies::username_password::{
            AuthorityParams,
            helpers::{raw_password_hash, verify_password},
        },
        test_support::spawn_webhook,
    };

    const PASSWORD_SALT: &str = "unit-test-password-salt";
    const PASSWORD_PEPPER: &str = "unit-test-password-pepper";
    const PASSWORD: &str = "register-me-please";
    const JWT_TTL_SECS: u64 = 900;
    const REFRESH_TTL_SECS: u64 = 86_400;
    const TOTP_TTL_SECS: u64 = 600;
    const WEBHOOK_KEY: &str = "unit-test-webhook-key";

    static KEYS: LazyLock<(String, Vec<u8>)> = LazyLock::new(|| {
        let pair = KeyPair::new().expect("keypair");
        let b64 = pair.base64_encode();
        (
            String::from_utf8(b64.private.clone()).expect("utf8 b64"),
            pair.public.clone(),
        )
    });

    const CLIENT_KEY: Uuid = uuid::uuid!("3c1b2c3d-3333-4000-8000-000000000001");
    const AUTHORITY_ID: Uuid = uuid::uuid!("3c1b2c3d-3333-4000-8000-000000000002");
    const NEW_USER_ID: Uuid = uuid::uuid!("3c1b2c3d-3333-4000-8000-000000000003");
    const NEW_TOKEN_ID: Uuid = uuid::uuid!("3c1b2c3d-3333-4000-8000-000000000004");

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

    #[derive(Clone)]
    enum Strategy {
        UsernamePassword,
        Oauth2,
        SingleUseToken,
        Missing,
    }

    struct MockAuthority {
        strategy: Strategy,
        /// `Some` makes this a second-factor authority, and the url is the only
        /// way a code can ever reach the user.
        totp: Option<Url>,
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

            match self.strategy {
                Strategy::Missing => Ok(None),
                Strategy::UsernamePassword => {
                    Ok(Some(authority(
                        AuthorityStrategy::UsernamePassword,
                        AuthorityParams::new(PASSWORD_SALT.to_owned())
                            .as_json_value()
                            .expect("params"),
                        self.totp.clone(),
                    )))
                },
                Strategy::Oauth2 => {
                    Ok(Some(authority(
                        AuthorityStrategy::Oauth2,
                        JsonValue::new(oauth2_authority_params()),
                        self.totp.clone(),
                    )))
                },
                Strategy::SingleUseToken => {
                    Ok(Some(authority(
                        AuthorityStrategy::SingleUseToken,
                        JsonValue::new(json!({})),
                        self.totp.clone(),
                    )))
                },
            }
        }
    }

    fn authority(
        strategy: AuthorityStrategy,
        params: JsonValue,
        webhook: Option<Url>,
    ) -> Authority {
        Authority {
            id: AUTHORITY_ID,
            name: "registering-authority".to_owned(),
            client_key: CLIENT_KEY,
            status: AuthorityStatus::Enabled,
            strategy,
            settings: AuthoritySettings {
                jwt_ttl: Duration::from_secs(JWT_TTL_SECS),
                jwt_nbf_offset: NbfOffset::Disabled,
                refresh_token_ttl: Duration::from_secs(REFRESH_TTL_SECS),
                totp: match webhook {
                    Some(webhook) => TotpSettings::Enabled {
                        totp_ttl: Duration::from_secs(TOTP_TTL_SECS),
                        webhook,
                        webhook_key: WEBHOOK_KEY.to_owned(),
                    },
                    None => TotpSettings::Disabled,
                },
                entitlements_encoding: EntitlementsEncoding::Txt,
            },
            params,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn oauth2_authority_params() -> Value {
        json!({
            "exchange_url": "https://oauth2.example.com/token",
            "flavor": "Google",
            "oauth2_id": "sso-client-id",
            "oauth2_secret": "sso-client-secret",
            "profile_url": "https://profile.example.com/me",
            "scopes": "openid email profile",
            "redirect_url": "https://authorize.example.com/oauth",
            "client_base_url": "https://app.example.com",
            "redirect_uri": "https://app.example.com/cb",
        })
    }

    struct MockInsertUser {
        duplicate: bool,
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

            if self.duplicate {
                return Err(UserAlreadyExistsError::username(&Username(
                    req.username.clone(),
                )));
            }

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
            self.log.push(format!(
                "user_authority_params:{}",
                params.params.inner_value()
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
                    user: User {
                        id: NEW_USER_ID,
                        kind: UserKind::Human,
                        status: UserStatus::Enabled,
                        username: "new-user".to_owned(),
                        email: None,
                        first_name: None,
                        last_name: None,
                        profile: json!({}),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    },
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
                "insert_refresh:{}.{}.{}.{}",
                req.user_id,
                req.authority_id,
                req.expires_at.timestamp(),
                req.totp_validated
            ));

            Ok(RefreshToken {
                id: NEW_TOKEN_ID,
                user_id: req.user_id,
                authority_id: req.authority_id,
                totp_validated: req.totp_validated,
                expires_at: req.expires_at,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    /// Stands in for the secret store. The point of the capture is that registration
    /// mints a secret of its own — 32 random characters from the same generator the
    /// backfill uses — for the user it just stored; the secret itself never reaches
    /// the log.
    struct MockInsertTotpSecret {
        log: CallLog,
    }

    #[async_trait]
    impl InsertTotpSecretQuery for MockInsertTotpSecret {
        async fn insert_totp_secret(
            &self,
            req: &InsertTotpSecretParams,
        ) -> Result<CreateTotpSecretResponse, BoxedError> {
            self.log.push(format!(
                "insert_totp_secret:{}.{}",
                req.user_id,
                req.secret_key.len()
            ));

            Ok(CreateTotpSecretResponse { success: true })
        }
    }

    fn use_case(
        log: &CallLog,
        strategy: Strategy,
        duplicate: bool,
    ) -> RegisterUseCase<
        MockAuthority,
        MockInsertUser,
        MockInsertUserAuthority,
        MockPermissionTree,
        MockPrivateKeys,
        MockRefreshTokens,
        MockInsertTotpSecret,
    > {
        use_case_with_webhook(log, strategy, duplicate, None)
    }

    /// `use_case` with the authority's second-factor webhook, if it has one.
    fn use_case_with_webhook(
        log: &CallLog,
        strategy: Strategy,
        duplicate: bool,
        webhook: Option<Url>,
    ) -> RegisterUseCase<
        MockAuthority,
        MockInsertUser,
        MockInsertUserAuthority,
        MockPermissionTree,
        MockPrivateKeys,
        MockRefreshTokens,
        MockInsertTotpSecret,
    > {
        RegisterUseCase::new(
            MockAuthority {
                strategy,
                totp: webhook,
                log: log.clone(),
            },
            MockInsertUser {
                duplicate,
                log: log.clone(),
            },
            MockInsertUserAuthority { log: log.clone() },
            MockPermissionTree { log: log.clone() },
            MockPrivateKeys { log: log.clone() },
            MockRefreshTokens { log: log.clone() },
            MockInsertTotpSecret { log: log.clone() },
        )
    }

    #[tokio::test]
    async fn username_password_dispatch_hashes_the_password_and_persists_everything() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let log = CallLog::default();
        let case = use_case(&log, Strategy::UsernamePassword, false);

        let res = case
            .register(&RegisterParams {
                client_key: CLIENT_KEY,
                params: JsonValue::new(json!({
                    "username": "new-user",
                    "password": PASSWORD,
                    "password_confirmation": PASSWORD,
                    "email": "new@example.com",
                    "first_name": null,
                    "last_name": null,
                    "kind": null,
                })),
            })
            .await
            .expect("a well-formed registration must succeed");

        assert_eq!(res.user_id, NEW_USER_ID, "the stored user's id is returned");
        assert_eq!(res.refresh_token, NEW_TOKEN_ID);

        let ops = log.ops();
        assert!(
            ops.contains(&"insert_user:new-user.human".to_owned()),
            "the user is inserted with the registrar-default kind (human), got {ops:?}"
        );
        assert!(
            ops.contains(&format!(
                "insert_user_authority:{NEW_USER_ID}.{AUTHORITY_ID}.new-user"
            )),
            "the user authority links the stored user to the authority, keyed by username"
        );

        let captured = ops
            .iter()
            .find(|op| op.starts_with("user_authority_params:"))
            .expect("authority params captured");
        let stored: Value =
            serde_json::from_str(captured.trim_start_matches("user_authority_params:"))
                .expect("captured params are compact json");
        let stored_hash = stored["password_hash"]
            .as_str()
            .expect("params carry the password_hash");
        assert!(
            stored_hash.starts_with("$argon2"),
            "the stored params must hold the argon2 hash, got {stored_hash}"
        );
        assert_eq!(
            verify_password(
                raw_password_hash(PASSWORD, PASSWORD_SALT, PASSWORD_PEPPER),
                stored_hash.to_owned(),
            ),
            Ok(true),
            "the persisted hash must verify the registered password"
        );

        let claims = Jwt::decode(&res.jwt, &KEYS.1).expect("jwt verifies");
        assert_eq!(claims.sub, Some(NEW_USER_ID));
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

        let insert = ops
            .iter()
            .find(|op| op.starts_with("insert_refresh:"))
            .expect("refresh token inserted");
        assert!(insert.starts_with(&format!("insert_refresh:{NEW_USER_ID}.{AUTHORITY_ID}.")));
    }

    #[tokio::test]
    async fn oauth2_dispatch_maps_the_profile_to_a_passwordless_user_authority() {
        let log = CallLog::default();
        let case = use_case(&log, Strategy::Oauth2, false);

        let res = case
            .register(&RegisterParams {
                client_key: CLIENT_KEY,
                params: JsonValue::new(json!({
                    "username": "profile@example.com",
                    "email": "profile@example.com",
                    "first_name": "Ann",
                    "last_name": "Last",
                    "kind": "human",
                })),
            })
            .await
            .expect("oauth2 registrations must succeed");

        assert_eq!(res.user_id, NEW_USER_ID);

        let ops = log.ops();
        assert!(
            ops.contains(&"insert_user:profile@example.com.human".to_owned()),
            "profile username and human kind must reach the user insert, got {ops:?}"
        );
        let auth_params = ops
            .iter()
            .find(|op| op.starts_with("user_authority_params:"))
            .expect("authority params captured");
        assert!(
            auth_params.trim_start_matches("user_authority_params:") == "null",
            "oauth2 user authorities carry no password material, got {auth_params}"
        );
    }

    #[tokio::test]
    async fn confirmation_mismatch_stops_before_any_write() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let log = CallLog::default();
        let case = use_case(&log, Strategy::UsernamePassword, false);

        let err = case
            .register(&RegisterParams {
                client_key: CLIENT_KEY,
                params: JsonValue::new(json!({
                    "username": "new-user",
                    "password": "one",
                    "password_confirmation": "two",
                    "email": null,
                    "first_name": null,
                    "last_name": null,
                    "kind": null,
                })),
            })
            .await
            .expect_err("mismatched confirmations must error");

        assert_eq!(
            err.to_string(),
            "password and password confirmation do not match"
        );
        let ops = log.ops();
        assert_eq!(ops.len(), 1, "only the authority lookup may run: {ops:?}");
    }

    #[tokio::test]
    async fn duplicate_username_propagates_without_writing_a_user_authority() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let log = CallLog::default();
        let case = use_case(&log, Strategy::UsernamePassword, true);

        let err = case
            .register(&RegisterParams {
                client_key: CLIENT_KEY,
                params: JsonValue::new(json!({
                    "username": "new-user",
                    "password": PASSWORD,
                    "password_confirmation": PASSWORD,
                    "email": null,
                    "first_name": null,
                    "last_name": null,
                    "kind": null,
                })),
            })
            .await
            .expect_err("a duplicate username must error");

        // the register flow surfaces the domain error verbatim — no re-wrapping,
        // and the sanitized copy is what reaches the wire
        let already_exists = err
            .downcast_ref::<UserAlreadyExistsError>()
            .expect("a duplicate username must propagate the typed error");
        assert_eq!(already_exists.username.0, "new-user");
        assert!(
            err.to_string()
                .contains("username is already taken"),
            "got {err}"
        );
        let ops = log.ops();
        assert!(
            !ops.iter()
                .any(|op| op.starts_with("insert_user_authority")),
            "no dangling user authority may be written for a rejected user: {ops:?}"
        );
        assert!(
            !ops.iter()
                .any(|op| op == "private_key")
        );
    }

    #[tokio::test]
    async fn unknown_client_key_aborts_before_the_registrar_runs() {
        let log = CallLog::default();
        let case = use_case(&log, Strategy::Missing, false);

        let err = case
            .register(&RegisterParams {
                client_key: CLIENT_KEY,
                params: JsonValue::new(json!({})),
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
    #[should_panic(expected = "not implemented")]
    async fn single_use_token_strategy_is_not_implemented() {
        let log = CallLog::default();
        let case = use_case(&log, Strategy::SingleUseToken, false);

        let _ = case
            .register(&RegisterParams {
                client_key: CLIENT_KEY,
                params: JsonValue::new(json!({})),
            })
            .await;
    }

    /// A TOTP authority exists to require the code, so a fresh account must not walk
    /// out with a session: it gets the code-only jwt, a pending refresh token, and a
    /// secret of its own — otherwise the code it is being made to wait for could
    /// never be computed, and the account would be dead on arrival.
    #[tokio::test]
    async fn registering_on_a_totp_authority_enrols_a_secret_and_holds_the_session_hostage() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);
        let (webhook, bodies) = spawn_webhook(r#"{"success":true}"#);

        let log = CallLog::default();
        let case = use_case_with_webhook(&log, Strategy::UsernamePassword, false, Some(webhook));

        let res = case
            .register(&RegisterParams {
                client_key: CLIENT_KEY,
                params: JsonValue::new(json!({
                    "username": "totp-new-user",
                    "password": PASSWORD,
                    "password_confirmation": PASSWORD,
                    "email": "totp@example.com",
                    "first_name": "Tot",
                    "last_name": "P",
                    "kind": null,
                })),
            })
            .await
            .expect("a registration whose webhook answers must succeed");

        let ops = log.ops();
        assert!(
            ops.contains(&format!("insert_totp_secret:{NEW_USER_ID}.32")),
            "a 32-character secret is enrolled for the new user, got {ops:?}"
        );
        let secret_at = ops
            .iter()
            .position(|op| op.starts_with("insert_totp_secret:"))
            .expect("secret enrolled");
        let refresh_at = ops
            .iter()
            .position(|op| op.starts_with("insert_refresh:"))
            .expect("refresh token inserted");
        assert!(
            secret_at < refresh_at,
            "the secret is stored before the session is handed out"
        );

        let body = bodies
            .lock()
            .expect("bodies")
            .first()
            .expect("the customer webhook must be called")
            .clone();
        let sent: Value = serde_json::from_str(&body).expect("webhook payload is json");
        assert_eq!(sent["webhook_key"], json!(WEBHOOK_KEY));
        assert_eq!(sent["email"], json!("totp@example.com"));
        assert_eq!(sent["name"], json!("Tot P"));
        assert_eq!(
            sent["code"]
                .as_str()
                .expect("a code")
                .len(),
            6,
            "the code is the short digit string the customer mails out"
        );

        let claims = Jwt::decode(&res.jwt, &KEYS.1).expect("jwt verifies");
        assert_eq!(
            claims
                .entitlements
                .expect("entitlements")
                .as_vec()
                .expect("txt"),
            vec![TOTP_VALIDATE_PERMISSION.to_owned()],
            "the permission tree waits for the code"
        );
        let iat = claims
            .iat
            .expect("iat is set");
        assert_eq!(claims.exp - iat, TOTP_TTL_SECS as usize);

        let insert = ops
            .iter()
            .find(|op| op.starts_with("insert_refresh:"))
            .expect("refresh token inserted");
        let parts: Vec<_> = insert.split('.').collect();
        assert_eq!(parts[3], "false", "the session is half-open until the code lands");
        let exp_at: i64 = parts[2]
            .parse()
            .expect("epoch");
        assert!(
            (exp_at - (Utc::now().timestamp() + TOTP_TTL_SECS as i64)).abs() <= 2,
            "the pending token dies with its code, not on the refresh ttl"
        );
    }

    /// No email means no code, and a TOTP authority with no code to send is not a
    /// login: the registration has to fail rather than quietly hand out a
    /// password-only session on an authority configured to demand more.
    #[tokio::test]
    async fn registering_without_an_email_on_a_totp_authority_mints_no_session() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);
        let (webhook, _bodies) = spawn_webhook(r#"{"success":true}"#);

        let log = CallLog::default();
        let case = use_case_with_webhook(&log, Strategy::UsernamePassword, false, Some(webhook));

        let err = case
            .register(&RegisterParams {
                client_key: CLIENT_KEY,
                params: JsonValue::new(json!({
                    "username": "no-email",
                    "password": PASSWORD,
                    "password_confirmation": PASSWORD,
                    "email": null,
                    "first_name": null,
                    "last_name": null,
                    "kind": null,
                })),
            })
            .await
            .expect_err("a code that cannot be delivered cannot open a session");

        assert!(
            err.to_string()
                .contains("unable to send totp code - missing email"),
            "got {err}"
        );

        let ops = log.ops();
        assert!(
            ops.iter()
                .any(|op| op.starts_with("insert_totp_secret:")),
            "the secret stays enrolled so a later login can retry the code, got {ops:?}"
        );
        assert!(
            !ops.iter()
                .any(|op| op.starts_with("insert_refresh:")),
            "no refresh token may exist for a session that never completed, got {ops:?}"
        );
    }
}
