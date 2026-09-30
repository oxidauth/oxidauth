use async_trait::async_trait;
use oxidauth_kernel::{
    authorities::{AuthorityNotFoundError, TotpSettings},
    error::BoxedError,
    totp_secrets::create_totp_secret::{CreateTotpSecret, CreateTotpSecretService},
    user_authorities::create_user_authority::*,
};
use oxidauth_repository::{
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    user_authorities::insert_user_authority::InsertUserAuthorityQuery,
};

use crate::auth::register::build_registrar;

pub struct CreateUserAuthorityUseCase<A, U>
where
    A: SelectAuthorityByClientKeyQuery,
    U: InsertUserAuthorityQuery,
{
    authority_by_client_key: A,
    insert_user_authority: U,
    totp_secrets: CreateTotpSecretService,
}

impl<A, U> CreateUserAuthorityUseCase<A, U>
where
    A: SelectAuthorityByClientKeyQuery,
    U: InsertUserAuthorityQuery,
{
    pub fn new(
        authority_by_client_key: A,
        insert_user_authority: U,
        totp_secrets: CreateTotpSecretService,
    ) -> Self {
        Self {
            authority_by_client_key,
            insert_user_authority,
            totp_secrets,
        }
    }
}

#[async_trait]
impl<A, U> CreateUserAuthorityServiceTrait for CreateUserAuthorityUseCase<A, U>
where
    A: SelectAuthorityByClientKeyQuery,
    U: InsertUserAuthorityQuery,
{
    #[tracing::instrument(name = "CreateUserAuthorityUseCase::create_user_authority", skip(self))]
    async fn create_user_authority(
        &self,
        params: &CreateUserAuthorityParams,
    ) -> Result<UserAuthority, BoxedError> {
        let authority = self
            .authority_by_client_key
            .select_authority_by_client_key(&params.into())
            .await?
            .ok_or_else(|| AuthorityNotFoundError::client_key(params.client_key))?;

        let registrar = build_registrar(&authority).await?;

        let user_authority = registrar
            .user_authority_from_request(params.params.clone())
            .await?;

        // If user's authority requires 2FA, ensure the user receives a totp secret
        if let TotpSettings::Enabled { .. } = authority.settings.totp {
            let totp_secret_params = CreateTotpSecret {
                user_id: params.user_id,
            };

            self.totp_secrets
                .create_totp_secret(&totp_secret_params)
                .await?;
        }

        self.insert_user_authority
            .call((params.user_id, &user_authority))
            .await
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
            find_authority_by_client_key::FindAuthorityByClientKey,
        },
        jwt::EntitlementsEncoding,
        totp_secrets::create_totp_secret::{
            CreateTotpSecretResponse,
            CreateTotpSecretServiceTrait,
        },
        user_authorities::UserAuthority,
    };
    use oxidauth_repository::user_authorities::insert_user_authority::InsertUserAuthority;
    use serde_json::{Value, json};
    use url::Url;
    use uuid::Uuid;

    use super::*;
    use crate::EnvGuard;

    const CLIENT_KEY: Uuid = uuid::uuid!("3c1b2c3d-3333-4000-8000-000000000001");
    const AUTHORITY_ID: Uuid = uuid::uuid!("3c1b2c3d-3333-4000-8000-000000000002");
    const USER_ID: Uuid = uuid::uuid!("3c1b2c3d-3333-4000-8000-000000000003");
    const PEPPER: &str = "unit-test-pepper";

    fn authority(totp_enabled: bool) -> Authority {
        Authority {
            id: AUTHORITY_ID,
            name: "default".to_owned(),
            client_key: CLIENT_KEY,
            status: AuthorityStatus::Enabled,
            strategy: AuthorityStrategy::UsernamePassword,
            settings: AuthoritySettings {
                jwt_ttl: Duration::from_secs(900),
                jwt_nbf_offset: Default::default(),
                refresh_token_ttl: Duration::from_secs(86_400),
                totp: if totp_enabled {
                    TotpSettings::Enabled {
                        totp_ttl: Duration::from_secs(30),
                        webhook: Url::parse("https://example.com/totp-hook").expect("url"),
                        webhook_key: "hook-key".to_owned(),
                    }
                } else {
                    TotpSettings::Disabled
                },
                entitlements_encoding: EntitlementsEncoding::Txt,
            },
            params: JsonValue::new(json!({ "password_salt": "unit-test-salt" })),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Snapshot of the (user_id, CreateUserAuthority) tuple the use case
    /// hands to the insert query after strategy-side hashing.
    #[derive(Debug)]
    struct CapturedInsert {
        user_id: Uuid,
        authority_id: Uuid,
        user_identifier: String,
        params: Value,
    }

    #[derive(Default)]
    struct Log {
        client_key_lookups: Vec<Uuid>,
        totp_secrets: Vec<Uuid>,
        inserts: Vec<CapturedInsert>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn snapshot(log: &SharedLog) -> (Vec<Uuid>, Vec<Uuid>, usize) {
        let log = log.lock().expect("log");
        (
            log.client_key_lookups.clone(),
            log.totp_secrets.clone(),
            log.inserts.len(),
        )
    }

    fn captured_inserts(log: &SharedLog) -> Vec<(Uuid, Uuid, String, Value)> {
        let log = log.lock().expect("log");
        log.inserts
            .iter()
            .map(|c| {
                (
                    c.user_id,
                    c.authority_id,
                    c.user_identifier.clone(),
                    c.params.clone(),
                )
            })
            .collect()
    }

    struct MockAuthorityLookup {
        log: SharedLog,
        found: bool,
        totp_enabled: bool,
    }

    #[async_trait]
    impl SelectAuthorityByClientKeyQuery for MockAuthorityLookup {
        async fn select_authority_by_client_key(
            &self,
            req: &FindAuthorityByClientKey,
        ) -> Result<Option<Authority>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .client_key_lookups
                .push(req.client_key);

            if self.found {
                Ok(Some(authority(self.totp_enabled)))
            } else {
                Ok(None)
            }
        }
    }

    struct MockInsert {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl InsertUserAuthorityQuery for MockInsert {
        async fn call(
            &self,
            params: impl Into<InsertUserAuthority> + Send + std::fmt::Debug + 'async_trait,
        ) -> Result<UserAuthority, BoxedError> {
            let params: InsertUserAuthority = params.into();

            self.log
                .lock()
                .expect("log")
                .inserts
                .push(CapturedInsert {
                    user_id: params.user_id,
                    authority_id: params.authority_id,
                    user_identifier: params.user_identifier.clone(),
                    params: params
                        .params
                        .clone()
                        .inner_value(),
                });

            if self.fail {
                return Err("simulated insert failure".into());
            }

            Ok(UserAuthority {
                user_id: params.user_id,
                authority_id: params.authority_id,
                user_identifier: params.user_identifier,
                params: JsonValue::new(params.params.inner_value()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    struct MockTotpSecrets {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl CreateTotpSecretServiceTrait for MockTotpSecrets {
        async fn create_totp_secret(
            &self,
            req: &CreateTotpSecret,
        ) -> Result<CreateTotpSecretResponse, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .totp_secrets
                .push(req.user_id);

            if self.fail {
                return Err("simulated totp secret failure".into());
            }

            Ok(CreateTotpSecretResponse { success: true })
        }
    }

    fn use_case(
        log: &SharedLog,
        found: bool,
        totp_enabled: bool,
        totp_fails: bool,
        insert_fails: bool,
    ) -> CreateUserAuthorityUseCase<MockAuthorityLookup, MockInsert> {
        CreateUserAuthorityUseCase::new(
            MockAuthorityLookup {
                log: log.clone(),
                found,
                totp_enabled,
            },
            MockInsert {
                log: log.clone(),
                fail: insert_fails,
            },
            Arc::new(MockTotpSecrets {
                log: log.clone(),
                fail: totp_fails,
            }),
        )
    }

    fn base_request() -> CreateUserAuthorityParams {
        CreateUserAuthorityParams {
            user_id: USER_ID,
            client_key: CLIENT_KEY,
            params: JsonValue::new(json!({
                "username": "octocat",
                "password": "register-me-please",
            })),
        }
    }

    #[tokio::test]
    async fn the_username_password_registrar_hashes_and_inserts_the_authority_row() {
        let _env = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PEPPER);
        let log = Arc::new(Mutex::new(Log::default()));

        let row = use_case(&log, true, false, false, false)
            .create_user_authority(&base_request())
            .await
            .expect("create succeeds");

        let (lookups, totp, inserts) = snapshot(&log);
        assert_eq!(lookups, vec![CLIENT_KEY]);
        assert!(totp.is_empty(), "totp disabled must not mint a secret");
        assert_eq!(inserts, 1);

        let (user_id, authority_id, identifier, params) = &captured_inserts(&log)[0];
        assert_eq!(*user_id, USER_ID);
        assert_eq!(
            *authority_id, AUTHORITY_ID,
            "the authority id comes from the looked-up row, not the request"
        );
        assert_eq!(identifier, "octocat");
        let hash = params["password_hash"]
            .as_str()
            .expect("password_hash stored by the strategy");
        assert!(
            hash.starts_with("$argon2"),
            "the password is hashed: {hash}"
        );
        assert!(!hash.contains("register-me-please"));
        assert_eq!(row.user_identifier, "octocat");
    }

    #[tokio::test]
    async fn an_unknown_client_key_becomes_an_authority_not_found_error() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, false, false, false, false)
            .create_user_authority(&base_request())
            .await
            .expect_err("Ok(None) from the client_key lookup is mapped");

        assert_eq!(
            error.to_string(),
            format!("authority not found by client_key: {CLIENT_KEY}")
        );
        // typed payload = `Box<AuthorityNotFoundError>` via the
        // `ok_or_else(...)?` `From` path (see find_user_by_username test)
        assert!(
            error
                .downcast_ref::<Box<AuthorityNotFoundError>>()
                .is_some()
        );
        let (_, totp, inserts) = snapshot(&log);
        assert!(totp.is_empty());
        assert_eq!(inserts, 0, "nothing is inserted for an unknown authority");
    }

    #[tokio::test]
    async fn a_two_fa_authority_mints_the_totp_secret_before_inserting() {
        let _env = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PEPPER);
        let log = Arc::new(Mutex::new(Log::default()));

        use_case(&log, true, true, false, false)
            .create_user_authority(&base_request())
            .await
            .expect("create succeeds");

        let (_, totp, inserts) = snapshot(&log);
        assert_eq!(
            totp,
            vec![USER_ID],
            "the secret is minted for the request's user"
        );
        assert_eq!(inserts, 1);
    }

    #[tokio::test]
    async fn a_failed_totp_secret_aborts_before_the_insert() {
        // without the row the user could never satisfy 2FA — the insert
        // must not proceed
        let _env = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PEPPER);
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true, true, true, false)
            .create_user_authority(&base_request())
            .await
            .expect_err("totp failure propagates");

        assert_eq!(error.to_string(), "simulated totp secret failure");
        assert_eq!(snapshot(&log).2, 0);
    }

    #[tokio::test]
    async fn params_the_strategy_rejects_are_not_inserted() {
        let _env = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PEPPER);
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();
        req.params = JsonValue::new(json!({ "unexpected": true }));

        let error = use_case(&log, true, false, false, false)
            .create_user_authority(&req)
            .await
            .expect_err("registrar rejection propagates");

        // the username_password strategy requires {username, password};
        // serde reports the first missing field, error unmapped
        assert_eq!(error.to_string(), "missing field `username`");
        assert_eq!(snapshot(&log).2, 0);
    }

    #[tokio::test]
    async fn insert_error_surfaces_unmapped() {
        let _env = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PEPPER);
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true, false, false, true)
            .create_user_authority(&base_request())
            .await
            .expect_err("duplicate-pair insert failure propagates");

        assert_eq!(error.to_string(), "simulated insert failure");
    }
}
