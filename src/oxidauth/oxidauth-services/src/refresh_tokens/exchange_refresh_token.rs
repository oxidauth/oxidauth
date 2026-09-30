use std::time::SystemTime;

use async_trait::async_trait;
use base64::prelude::{BASE64_STANDARD, Engine};
use chrono::DateTime;
use oxidauth_kernel::{
    auth::{authenticate::AuthenticateResponse, tree::PermissionSearch},
    authorities::{NbfOffset, find_authority_by_id::FindAuthorityById},
    error::BoxedError,
    jwt::{DurationDirection, Jwt, epoch_from_now, epoch_from_time},
    private_keys::find_most_recent_private_key::FindMostRecentPrivateKey,
    refresh_tokens::{
        create_refresh_token::CreateRefreshToken,
        delete_refresh_token_by_id::DeleteRefreshTokenById,
        exchange_refresh_token::*,
        find_refresh_token_by_id::FindRefreshTokenById,
    },
    user_authorities::find_user_authority_by_user_id_and_authority_id::FindUserAuthorityByUserIdAndAuthorityId,
    users::{UserStatus, find_user_by_id::FindUserById},
};
use oxidauth_repository::{
    auth::tree::PermissionTreeQuery,
    authorities::select_authority_by_id::SelectAuthorityByIdQuery,
    private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery,
    refresh_tokens::{
        delete_refresh_token_by_id::DeleteRefreshTokenByIdQuery,
        insert_refresh_token::InsertRefreshTokenQuery,
        select_refresh_token_by_id::SelectRefreshTokenByIdQuery,
    },
    user_authorities::select_user_authority_by_user_id_and_authority_id::SelectUserAuthorityByUserIdAndAuthorityIdQuery,
    users::select_user_by_id_query::SelectUserByIdQuery,
};

pub struct ExchangeRefreshTokenUseCase<T, I, U, A, P, K, D, UU>
where
    T: SelectRefreshTokenByIdQuery,
    I: InsertRefreshTokenQuery,
    U: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
    A: SelectAuthorityByIdQuery,
    P: PermissionTreeQuery,
    K: SelectMostRecentPrivateKeyQuery,
    D: DeleteRefreshTokenByIdQuery,
    UU: SelectUserByIdQuery,
{
    refresh_tokens: T,
    insert_refresh_tokens: I,
    user_authorities: U,
    authorities: A,
    permission_tree: P,
    private_keys: K,
    delete_refresh_tokens: D,
    user_by_id: UU,
}

impl<T, I, U, A, P, K, D, UU> ExchangeRefreshTokenUseCase<T, I, U, A, P, K, D, UU>
where
    T: SelectRefreshTokenByIdQuery,
    I: InsertRefreshTokenQuery,
    U: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
    A: SelectAuthorityByIdQuery,
    P: PermissionTreeQuery,
    K: SelectMostRecentPrivateKeyQuery,
    D: DeleteRefreshTokenByIdQuery,
    UU: SelectUserByIdQuery,
{
    pub fn new(
        refresh_tokens: T,
        insert_refresh_tokens: I,
        user_authorities: U,
        authorities: A,
        permission_tree: P,
        private_keys: K,
        delete_refresh_tokens: D,
        user_by_id: UU,
    ) -> Self {
        Self {
            refresh_tokens,
            insert_refresh_tokens,
            user_authorities,
            authorities,
            permission_tree,
            private_keys,
            delete_refresh_tokens,
            user_by_id,
        }
    }
}

#[async_trait]
impl<T, I, U, A, P, K, D, UU> ExchangeRefreshTokenServiceTrait
    for ExchangeRefreshTokenUseCase<T, I, U, A, P, K, D, UU>
where
    T: SelectRefreshTokenByIdQuery,
    I: InsertRefreshTokenQuery,
    U: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
    A: SelectAuthorityByIdQuery,
    P: PermissionTreeQuery,
    K: SelectMostRecentPrivateKeyQuery,
    D: DeleteRefreshTokenByIdQuery,
    UU: SelectUserByIdQuery,
{
    #[tracing::instrument(
        name = "ExchangeRefreshTokenUseCase::exchange_refresh_token",
        skip(self)
    )]
    async fn exchange_refresh_token(
        &self,
        req: &ExchangeRefreshToken,
    ) -> Result<AuthenticateResponse, BoxedError> {
        let RefreshToken {
            user_id,
            authority_id,
            expires_at,
            ..
        } = self
            .refresh_tokens
            .select_refresh_token_by_id(&FindRefreshTokenById {
                refresh_token_id: req.refresh_token,
            })
            .await?;

        let now = epoch_from_time(SystemTime::now())
            .map_err(|err| format!("error getting epoch time from system time: {:?}", err))?;

        if expires_at.timestamp() < now as i64 {
            self.delete_refresh_tokens
                .delete_refresh_token_by_id(&DeleteRefreshTokenById {
                    refresh_token_id: req.refresh_token,
                })
                .await?;
            return Err("refresh token has expired".into());
        }

        // OXA-000009 (policy A): disabled accounts stop renewing. This path
        // never loaded the user, so a disabled session rotated forever; one
        // user read per refresh gates it before any token material is
        // consulted. The expired-token hygiene above still applies; for a
        // disabled-but-valid token the old row is deliberately NOT deleted or
        // rotated (nothing is minted, and the row stays auditable — see
        // changelog).
        let user = self
            .user_by_id
            .select_user_by_id(&FindUserById { user_id })
            .await?;

        if matches!(user.status, UserStatus::Disabled) {
            tracing::warn!(user_id = %user.id, "refresh refused: account is disabled");
            return Err("account is disabled".into());
        }

        let authority_id = self
            .user_authorities
            .select_user_authority_by_user_id_and_authority_id(
                &FindUserAuthorityByUserIdAndAuthorityId {
                    user_id,
                    authority_id,
                },
            )
            .await?
            .authority
            .id;

        let authority = self
            .authorities
            .select_authority_by_id(&FindAuthorityById { authority_id })
            .await?;

        let permissions = self
            .permission_tree
            .permission_tree(&PermissionSearch::User(user_id))
            .await?
            .permissions;

        let private_key = self
            .private_keys
            .select_most_recent_private_key(&FindMostRecentPrivateKey {})
            .await?;

        let private_key = BASE64_STANDARD.decode(private_key.private_key)?;

        let mut jwt_builder = Jwt::builder()
            .with_subject(user_id)
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
            .insert_refresh_tokens
            .insert_refresh_token(&CreateRefreshToken {
                user_id,
                authority_id: authority.id,
                expires_at: refresh_token_exp_at,
            })
            .await?;

        self.delete_refresh_tokens
            .delete_refresh_token_by_id(&DeleteRefreshTokenById {
                refresh_token_id: req.refresh_token,
            })
            .await?;

        Ok(AuthenticateResponse {
            jwt,
            refresh_token: refresh_token.id,
            user_id,
        })
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
        user_authorities::{UserAuthority, UserAuthorityWithAuthority},
        users::{User, UserKind, UserStatus},
    };
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    const JWT_TTL_SECS: u64 = 900;
    const REFRESH_TTL_SECS: u64 = 86_400;
    const NBF_OFFSET_SECS: u64 = 30;

    static KEYS: LazyLock<(String, Vec<u8>)> = LazyLock::new(|| {
        let pair = KeyPair::new().expect("keypair");
        let b64 = pair.base64_encode();
        (
            String::from_utf8(b64.private.clone()).expect("utf8 b64"),
            pair.public.clone(),
        )
    });

    const TOKEN_ID: Uuid = uuid::uuid!("0a1b2c3d-0000-4000-8000-000000000001");
    const USER_ID: Uuid = uuid::uuid!("0a1b2c3d-0000-4000-8000-000000000002");
    const TOKEN_AUTHORITY_ID: Uuid = uuid::uuid!("0a1b2c3d-0000-4000-8000-000000000003");
    const UA_AUTHORITY_ID: Uuid = uuid::uuid!("0a1b2c3d-0000-4000-8000-000000000004");
    const NEW_TOKEN_ID: Uuid = uuid::uuid!("0a1b2c3d-0000-4000-8000-000000000005");

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

        fn taken(&self, op: &str) -> usize {
            self.ops()
                .into_iter()
                .filter(|entry| entry.starts_with(op))
                .count()
        }
    }

    struct MockSelectToken {
        expired: bool,
        fail: bool,
        log: CallLog,
    }

    #[async_trait]
    impl SelectRefreshTokenByIdQuery for MockSelectToken {
        async fn select_refresh_token_by_id(
            &self,
            req: &FindRefreshTokenById,
        ) -> Result<RefreshToken, BoxedError> {
            self.log
                .push(format!("select_token:{}", req.refresh_token_id));

            if self.fail {
                return Err("simulated select failure".into());
            }

            let offset = if self.expired {
                -5
            } else {
                3_600
            };

            Ok(RefreshToken {
                id: req.refresh_token_id,
                user_id: USER_ID,
                authority_id: TOKEN_AUTHORITY_ID,
                expires_at: Utc::now() + chrono::Duration::try_seconds(offset).expect("valid"),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    struct MockInsertToken {
        fail: bool,
        log: CallLog,
    }

    #[async_trait]
    impl InsertRefreshTokenQuery for MockInsertToken {
        async fn insert_refresh_token(
            &self,
            req: &CreateRefreshToken,
        ) -> Result<RefreshToken, BoxedError> {
            self.log
                .push("insert_token".to_owned());

            if self.fail {
                return Err("simulated insert failure".into());
            }

            self.log.push(format!(
                "insert:{}.{}.{}",
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

    struct MockSelectUserAuthority {
        fail: bool,
        log: CallLog,
    }

    #[async_trait]
    impl SelectUserAuthorityByUserIdAndAuthorityIdQuery for MockSelectUserAuthority {
        async fn select_user_authority_by_user_id_and_authority_id(
            &self,
            req: &FindUserAuthorityByUserIdAndAuthorityId,
        ) -> Result<UserAuthorityWithAuthority, BoxedError> {
            self.log.push(format!(
                "select_user_authority:{}.{}",
                req.user_id, req.authority_id
            ));

            if self.fail {
                return Err("simulated user-authority failure".into());
            }

            Ok(UserAuthorityWithAuthority {
                user_authority: UserAuthority {
                    user_id: req.user_id,
                    authority_id: req.authority_id,
                    user_identifier: "rotator".to_owned(),
                    params: JsonValue::empty(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                authority: authority(UA_AUTHORITY_ID, false),
            })
        }
    }

    struct MockSelectAuthority {
        fail: bool,
        nbf: bool,
        log: CallLog,
    }

    #[async_trait]
    impl SelectAuthorityByIdQuery for MockSelectAuthority {
        async fn select_authority_by_id(
            &self,
            req: &FindAuthorityById,
        ) -> Result<Authority, BoxedError> {
            self.log
                .push(format!("select_authority:{}", req.authority_id));

            if self.fail {
                return Err("simulated authority failure".into());
            }

            Ok(authority(req.authority_id, self.nbf))
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
            self.log
                .push(format!("tree:{user_id}"));

            Ok(PermissionsResponse {
                tree: PermissionTree::User(UserNode {
                    user: user(user_id),
                    roles: vec![],
                    permissions: vec![],
                }),
                permissions: vec!["oxidauth:**:**".to_owned(), "app:read:all".to_owned()],
            })
        }
    }

    struct MockPrivateKeys {
        private_key: Vec<u8>,
        fail: bool,
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

            if self.fail {
                return Err("simulated private key failure".into());
            }

            Ok(PrivateKey {
                id: Uuid::new_v4(),
                private_key: self.private_key.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    struct MockDeleteToken {
        fail: bool,
        log: CallLog,
    }

    #[async_trait]
    impl DeleteRefreshTokenByIdQuery for MockDeleteToken {
        async fn delete_refresh_token_by_id(
            &self,
            req: &DeleteRefreshTokenById,
        ) -> Result<RefreshToken, BoxedError> {
            self.log
                .push(format!("delete_token:{}", req.refresh_token_id));

            if self.fail {
                return Err("simulated delete failure".into());
            }

            Ok(RefreshToken {
                id: req.refresh_token_id,
                user_id: USER_ID,
                authority_id: TOKEN_AUTHORITY_ID,
                expires_at: Utc::now(),
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

    fn authority(id: Uuid, nbf_enabled: bool) -> Authority {
        Authority {
            id,
            name: "rotating-authority".to_owned(),
            client_key: Uuid::new_v4(),
            status: AuthorityStatus::Enabled,
            strategy: AuthorityStrategy::UsernamePassword,
            settings: AuthoritySettings {
                jwt_ttl: Duration::from_secs(JWT_TTL_SECS),
                jwt_nbf_offset: if nbf_enabled {
                    NbfOffset::Enabled(Duration::from_secs(NBF_OFFSET_SECS))
                } else {
                    NbfOffset::Disabled
                },
                refresh_token_ttl: Duration::from_secs(REFRESH_TTL_SECS),
                totp: TotpSettings::Disabled,
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
            username: "rotator".to_owned(),
            email: None,
            first_name: None,
            last_name: None,
            profile: json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Clone, Copy)]
    struct Fail {
        select: bool,
        insert: bool,
        user_authority: bool,
        authority: bool,
        tree: bool,
        key: bool,
        delete: bool,
        user: bool,
    }

    const NO_FAIL: Fail = Fail {
        select: false,
        insert: false,
        user_authority: false,
        authority: false,
        tree: false,
        key: false,
        delete: false,
        user: false,
    };

    fn use_case_with_key(
        log: &CallLog,
        expired: bool,
        fail: Fail,
        private_key: Vec<u8>,
        nbf: bool,
        user_status: UserStatus,
    ) -> ExchangeRefreshTokenUseCase<
        MockSelectToken,
        MockInsertToken,
        MockSelectUserAuthority,
        MockSelectAuthority,
        MockPermissionTree,
        MockPrivateKeys,
        MockDeleteToken,
        MockUserById,
    > {
        ExchangeRefreshTokenUseCase::new(
            MockSelectToken {
                expired,
                fail: fail.select,
                log: log.clone(),
            },
            MockInsertToken {
                fail: fail.insert,
                log: log.clone(),
            },
            MockSelectUserAuthority {
                fail: fail.user_authority,
                log: log.clone(),
            },
            MockSelectAuthority {
                fail: fail.authority,
                nbf,
                log: log.clone(),
            },
            MockPermissionTree {
                fail: fail.tree,
                log: log.clone(),
            },
            MockPrivateKeys {
                private_key,
                fail: fail.key,
                log: log.clone(),
            },
            MockDeleteToken {
                fail: fail.delete,
                log: log.clone(),
            },
            MockUserById {
                status: user_status,
                fail: fail.user,
                log: log.clone(),
            },
        )
    }

    fn use_case(log: &CallLog, expired: bool, fail: Fail) -> impl ExchangeRefreshTokenServiceTrait {
        use_case_status(log, expired, fail, UserStatus::Enabled)
    }

    fn use_case_status(
        log: &CallLog,
        expired: bool,
        fail: Fail,
        user_status: UserStatus,
    ) -> impl ExchangeRefreshTokenServiceTrait {
        use_case_with_key(
            log,
            expired,
            fail,
            KEYS.0.clone().into_bytes(),
            false,
            user_status,
        )
    }

    fn req() -> ExchangeRefreshToken {
        ExchangeRefreshToken {
            refresh_token: TOKEN_ID,
        }
    }

    fn insert_capture(log: &CallLog) -> String {
        log.ops()
            .into_iter()
            .find(|entry| entry.starts_with("insert:"))
            .expect("an insert capture must exist")
    }

    #[tokio::test]
    async fn expired_token_is_deleted_by_id_and_reported_as_expired() {
        let log = CallLog::default();
        let case = use_case(&log, true, NO_FAIL);

        let err = case
            .exchange_refresh_token(&req())
            .await
            .expect_err("an expired token must be rejected");

        assert_eq!(err.to_string(), "refresh token has expired");

        let ops = log.ops();
        assert_eq!(
            ops,
            vec![
                format!("select_token:{TOKEN_ID}"),
                format!("delete_token:{TOKEN_ID}"),
            ],
            "the expired token must be deleted by its own id and nothing else may run"
        );
    }

    #[tokio::test]
    async fn rotation_inserts_the_new_token_before_deleting_the_old_one() {
        let log = CallLog::default();
        let case = use_case(&log, false, NO_FAIL);

        let res = case
            .exchange_refresh_token(&req())
            .await
            .expect("a valid token must rotate");

        assert_eq!(
            res.user_id, USER_ID,
            "the response carries the token's owner"
        );
        assert_eq!(
            res.refresh_token, NEW_TOKEN_ID,
            "the response carries the freshly inserted token id"
        );

        // authority resolution is keyed by the *user authority's* authority,
        // not the stale authority id baked into the old refresh token
        assert!(
            log.ops().contains(&format!(
                "select_user_authority:{USER_ID}.{TOKEN_AUTHORITY_ID}"
            )),
            "the user authority must be looked up with (token user, token authority)"
        );
        assert!(
            log.ops()
                .contains(&format!("select_authority:{UA_AUTHORITY_ID}")),
            "pinned: the authority fetched is the one attached to the user authority, \
             so the token's own authority_id is overridden"
        );
        assert!(
            !log.ops()
                .contains(&format!("select_authority:{TOKEN_AUTHORITY_ID}"))
        );

        let capture = insert_capture(&log);
        let parts: Vec<_> = capture.split('.').collect();
        assert_eq!(parts[0], format!("insert:{USER_ID}"));
        assert_eq!(parts[1], UA_AUTHORITY_ID.to_string());
        let exp_at: i64 = parts[2]
            .parse()
            .expect("epoch in insert capture");
        let now = Utc::now().timestamp();
        assert!(
            (exp_at - (now + REFRESH_TTL_SECS as i64)).abs() <= 2,
            "the new token expires one authority refresh_ttl from now, got {exp_at}"
        );

        let ops = log.ops();
        let insert_at = ops
            .iter()
            .position(|op| op == "insert_token")
            .expect("insert ran");
        let delete_at = ops
            .iter()
            .position(|op| op == &format!("delete_token:{TOKEN_ID}"))
            .expect("the old token must be deleted on rotation");
        assert!(
            insert_at < delete_at,
            "pinned order: the replacement is inserted before the old token is deleted \
             (a crash between the two keeps the session alive), got {ops:?}"
        );
    }

    #[tokio::test]
    async fn disabled_user_cannot_rotate_a_still_valid_refresh_token() {
        // OXA-000009 (policy A): a disabled account's sessions stop renewing.
        // The gate fires after the expiry branch and before the user-authority
        // lookup: no jwt is signed, no replacement token is inserted, and the
        // still-valid old token is neither rotated nor wiped (it stays
        // auditable).
        let log = CallLog::default();
        let case = use_case_status(&log, false, NO_FAIL, UserStatus::Disabled);

        let err = case
            .exchange_refresh_token(&req())
            .await
            .expect_err("a disabled account must not renew its session");

        assert_eq!(err.to_string(), "account is disabled");
        assert_eq!(
            log.ops(),
            vec![
                format!("select_token:{TOKEN_ID}"),
                format!("find_user:{USER_ID}"),
            ],
            "pinned order: the status gate runs before the user-authority, tree, \
             key, insert, and delete legs — no rotation, no token wipe"
        );
    }

    #[tokio::test]
    async fn invited_users_keep_renewing_under_the_status_gate() {
        // OXA-000009 Step 0 decision: only `Disabled` fails closed; `Invited`
        // accounts keep their sessions (meaningful `Invited` semantics is a
        // separate product item).
        let log = CallLog::default();
        let case = use_case_status(&log, false, NO_FAIL, UserStatus::Invited);

        let res = case
            .exchange_refresh_token(&req())
            .await
            .expect("an invited account still rotates");

        assert_eq!(res.refresh_token, NEW_TOKEN_ID);
    }

    #[tokio::test]
    async fn jwt_claims_carry_subject_issuer_authority_ttl_and_tree_entitlements() {
        let log = CallLog::default();
        let case = use_case(&log, false, NO_FAIL);

        let res = case
            .exchange_refresh_token(&req())
            .await
            .expect("a valid token must rotate");

        let claims =
            Jwt::decode(&res.jwt, &KEYS.1).expect("the jwt must verify against the keypair");

        assert_eq!(claims.sub, Some(USER_ID), "subject is the token owner");
        assert_eq!(claims.iss.as_deref(), Some("oxidauth"), "issuer is fixed");
        let iat = claims
            .iat
            .expect("iat is set");
        assert_eq!(
            claims.exp - iat,
            JWT_TTL_SECS as usize,
            "ttl comes from the authority settings jwt_ttl"
        );
        assert_eq!(
            claims
                .entitlements
                .expect("entitlements")
                .as_vec()
                .expect("txt decodes"),
            vec!["oxidauth:**:**".to_owned(), "app:read:all".to_owned()],
            "entitlements come from the permission tree under the authority's txt encoding"
        );
        // BUG(pinned): `JwtBuilder::build` ALWAYS emits `nbf` with a
        // now-10s default; `NbfOffset::Disabled` on the authority only skips
        // the *custom* offset, it does not suppress the claim
        assert_eq!(
            claims
                .nbf
                .expect("nbf is emitted regardless"),
            iat - 10
        );
    }

    #[tokio::test]
    async fn nbf_offset_enabled_on_the_authority_is_emitted_in_the_jwt() {
        let log = CallLog::default();
        let case = use_case_with_key(
            &log,
            false,
            NO_FAIL,
            KEYS.0.clone().into_bytes(),
            true,
            UserStatus::Enabled,
        );

        let res = case
            .exchange_refresh_token(&req())
            .await
            .expect("a valid token must rotate");

        let claims = Jwt::decode(&res.jwt, &KEYS.1).expect("the jwt must decode");

        // both iat and nbf derive from the same clock reading inside build(),
        // so the offset is exact (mirrors the kernel jwt suite)
        let iat = claims
            .iat
            .expect("iat is set");
        assert_eq!(
            claims
                .nbf
                .expect("nbf is emitted"),
            iat - NBF_OFFSET_SECS as usize,
            "nbf is the authority's jwt_nbf_offset before iat"
        );
    }

    #[tokio::test]
    async fn every_repository_failure_propagates() {
        let cases: Vec<(Fail, &str)> = vec![
            (
                Fail {
                    select: true,
                    ..NO_FAIL
                },
                "simulated select failure",
            ),
            (
                Fail {
                    user_authority: true,
                    ..NO_FAIL
                },
                "simulated user-authority failure",
            ),
            (
                Fail {
                    authority: true,
                    ..NO_FAIL
                },
                "simulated authority failure",
            ),
            (
                Fail {
                    tree: true,
                    ..NO_FAIL
                },
                "simulated permission tree failure",
            ),
            (
                Fail {
                    key: true,
                    ..NO_FAIL
                },
                "simulated private key failure",
            ),
            (
                Fail {
                    insert: true,
                    ..NO_FAIL
                },
                "simulated insert failure",
            ),
            (
                Fail {
                    delete: true,
                    ..NO_FAIL
                },
                "simulated delete failure",
            ),
            (
                Fail {
                    user: true,
                    ..NO_FAIL
                },
                "simulated user lookup failure",
            ),
        ];

        for (fail, message) in cases {
            let log = CallLog::default();
            let case = use_case(&log, false, fail);

            let err = case
                .exchange_refresh_token(&req())
                .await
                .expect_err("repository failures must propagate");

            assert!(
                err.to_string()
                    .contains(message),
                "expected {message:?}, got {err}"
            );
        }
    }

    #[tokio::test]
    async fn malformed_base64_private_key_fails_before_inserting_a_token() {
        let log = CallLog::default();
        let case = use_case_with_key(
            &log,
            false,
            NO_FAIL,
            b"!!!not base64!!!".to_vec(),
            false,
            UserStatus::Enabled,
        );

        case.exchange_refresh_token(&req())
            .await
            .expect_err("a corrupt stored key must abort the exchange");

        assert_eq!(
            log.taken("insert_token"),
            0,
            "no replacement token may exist when the jwt cannot be signed"
        );
        assert_eq!(
            log.taken("delete_token"),
            0,
            "the still-valid old token must survive a signing failure"
        );
        assert_eq!(
            log.taken("private_key"),
            1,
            "the key was fetched exactly once before the abort"
        );
    }
}
