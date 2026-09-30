use async_trait::async_trait;
use base64::prelude::*;
use chrono::DateTime;
use oxidauth_kernel::{
    auth::{
        Registrar,
        register::{RegisterParams, RegisterResponse, RegisterServiceTrait},
    },
    authorities::{Authority, AuthorityNotFoundError, AuthorityStrategy, NbfOffset},
    error::BoxedError,
    jwt::{DurationDirection, Jwt, epoch_from_now},
    private_keys::find_most_recent_private_key::FindMostRecentPrivateKey,
};
use oxidauth_repository::{
    auth::tree::{PermissionSearch, PermissionTreeQuery},
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery,
    refresh_tokens::insert_refresh_token::{CreateRefreshToken, InsertRefreshTokenQuery},
    user_authorities::insert_user_authority::InsertUserAuthorityQuery,
    users::insert_user::InsertUserQuery,
};

use crate::auth::strategies;

pub struct RegisterUseCase<T, U, A, P, M, R>
where
    T: SelectAuthorityByClientKeyQuery,
    U: InsertUserQuery,
    A: InsertUserAuthorityQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
{
    authority_by_client_key: T,
    users: U,
    user_authorities: A,
    permission_tree: P,
    private_keys: M,
    refresh_tokens: R,
}

impl<T, U, A, P, M, R> RegisterUseCase<T, U, A, P, M, R>
where
    T: SelectAuthorityByClientKeyQuery,
    U: InsertUserQuery,
    A: InsertUserAuthorityQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
{
    pub fn new(
        authority_by_client_key: T,
        users: U,
        user_authorities: A,
        permission_tree: P,
        private_keys: M,
        refresh_tokens: R,
    ) -> Self {
        Self {
            authority_by_client_key,
            users,
            user_authorities,
            permission_tree,
            private_keys,
            refresh_tokens,
        }
    }
}

#[async_trait]
impl<T, U, A, P, M, R> RegisterServiceTrait for RegisterUseCase<T, U, A, P, M, R>
where
    T: SelectAuthorityByClientKeyQuery,
    U: InsertUserQuery,
    A: InsertUserAuthorityQuery,
    P: PermissionTreeQuery,
    M: SelectMostRecentPrivateKeyQuery,
    R: InsertRefreshTokenQuery,
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

        // add default roles and permissions
        let permissions = self
            .permission_tree
            .permission_tree(&PermissionSearch::User(user.id))
            .await?
            .permissions;

        let private_key = self
            .private_keys
            .select_most_recent_private_key(&FindMostRecentPrivateKey {})
            .await?;

        let private_key = BASE64_STANDARD.decode(private_key.private_key)?;

        let mut jwt_builder = Jwt::builder()
            .with_subject(user.id)
            .with_issuer("oxidauth".to_owned())
            .with_expires_in(authority.settings.jwt_ttl)
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
                user_id: user.id,
                authority_id: authority.id,
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
    use uuid::Uuid;

    use super::*;
    use crate::{
        EnvGuard,
        auth::strategies::username_password::{
            AuthorityParams,
            helpers::{raw_password_hash, verify_password},
        },
    };

    const PASSWORD_SALT: &str = "unit-test-password-salt";
    const PASSWORD_PEPPER: &str = "unit-test-password-pepper";
    const PASSWORD: &str = "register-me-please";
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
                    )))
                },
                Strategy::Oauth2 => {
                    Ok(Some(authority(
                        AuthorityStrategy::Oauth2,
                        JsonValue::new(oauth2_authority_params()),
                    )))
                },
                Strategy::SingleUseToken => {
                    Ok(Some(authority(
                        AuthorityStrategy::SingleUseToken,
                        JsonValue::new(json!({})),
                    )))
                },
            }
        }
    }

    fn authority(strategy: AuthorityStrategy, params: JsonValue) -> Authority {
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
                totp: TotpSettings::Disabled,
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
    > {
        RegisterUseCase::new(
            MockAuthority {
                strategy,
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
}
