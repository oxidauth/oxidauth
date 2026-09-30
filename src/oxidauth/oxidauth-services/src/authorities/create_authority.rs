use async_trait::async_trait;
use oxidauth_kernel::{authorities::create_authority::*, error::BoxedError};
use oxidauth_repository::authorities::insert_authority::InsertAuthorityQuery;
use uuid::Uuid;

pub struct CreateAuthorityUseCase<T>
where
    T: InsertAuthorityQuery,
{
    authorities: T,
}

impl<T> CreateAuthorityUseCase<T>
where
    T: InsertAuthorityQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<T> CreateAuthorityServiceTrait for CreateAuthorityUseCase<T>
where
    T: InsertAuthorityQuery,
{
    #[tracing::instrument(name = "CreateAuthorityUseCase::create_authority", skip(self))]
    async fn create_authority(&self, req: &mut CreateAuthority) -> Result<Authority, BoxedError> {
        if req.client_key.is_none() {
            req.client_key
                .replace(Uuid::new_v4());
        }

        if req.status.is_none() {
            req.status
                .replace(AuthorityStatus::Enabled);
        }

        self.authorities
            .insert_authority(req)
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
    use oxidauth_kernel::{JsonValue, authorities::TotpSettings, jwt::EntitlementsEncoding};
    use serde_json::json;

    use super::*;

    fn authority_id() -> Uuid {
        uuid::uuid!("33333333-3333-4333-8333-333333333333")
    }

    fn settings() -> AuthoritySettings {
        AuthoritySettings {
            jwt_ttl: Duration::from_secs(900),
            jwt_nbf_offset: Default::default(),
            refresh_token_ttl: Duration::from_secs(86_400),
            totp: TotpSettings::Disabled,
            entitlements_encoding: EntitlementsEncoding::Txt,
        }
    }

    #[derive(Clone, Debug)]
    struct CapturedInsert {
        name: String,
        client_key: Option<Uuid>,
        status: Option<String>,
        strategy: String,
        params: serde_json::Value,
    }

    #[derive(Default)]
    struct Log {
        inserts: Vec<CapturedInsert>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn inserts(log: &SharedLog) -> Vec<CapturedInsert> {
        log.lock()
            .expect("log")
            .inserts
            .clone()
    }

    struct MockInsert {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl InsertAuthorityQuery for MockInsert {
        async fn insert_authority(&self, req: &CreateAuthority) -> Result<Authority, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .inserts
                .push(CapturedInsert {
                    name: req.name.clone(),
                    client_key: req.client_key,
                    status: req
                        .status
                        .as_ref()
                        .map(|status| status.to_string()),
                    strategy: req.strategy.to_string(),
                    params: req
                        .params
                        .clone()
                        .inner_value(),
                });

            if self.fail {
                return Err("simulated insert failure".into());
            }

            Ok(Authority {
                id: authority_id(),
                name: req.name.clone(),
                client_key: req
                    .client_key
                    .expect("client_key present after defaults"),
                status: req
                    .status
                    .as_ref()
                    .map_or(AuthorityStatus::Enabled, |status| {
                        status
                            .to_string()
                            .parse()
                            .expect("status")
                    }),
                strategy: req.strategy,
                settings: settings(),
                params: req.params.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> CreateAuthorityUseCase<MockInsert> {
        CreateAuthorityUseCase::new(MockInsert {
            log: log.clone(),
            fail,
        })
    }

    fn base_request() -> CreateAuthority {
        CreateAuthority {
            name: "new-authority".to_owned(),
            client_key: None,
            status: None,
            strategy: AuthorityStrategy::UsernamePassword,
            settings: settings(),
            params: JsonValue::new(json!({ "password_salt": "pepper" })),
        }
    }

    #[tokio::test]
    async fn omitted_client_key_is_generated_as_fresh_v4_uuid() {
        // Creation-side generation is intended behavior (a brand-new
        // authority has no existing logins to break — unlike the update
        // path, see update_authority::tests::omitted_client_key_is_regenerated).
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();
        req.client_key = None;

        use_case(&log, false)
            .create_authority(&mut req)
            .await
            .expect("create succeeds");

        let captured = inserts(&log);
        assert_eq!(captured.len(), 1);

        let generated = captured[0]
            .client_key
            .expect("a client_key was generated");
        assert_eq!(
            generated.get_version(),
            Some(uuid::Version::Random),
            "the generated client_key is a fresh v4 Uuid"
        );
    }

    #[tokio::test]
    async fn supplied_client_key_is_preserved_verbatim() {
        let log = Arc::new(Mutex::new(Log::default()));
        let supplied = uuid::Uuid::new_v4();
        let mut req = base_request();
        req.client_key = Some(supplied);

        use_case(&log, false)
            .create_authority(&mut req)
            .await
            .expect("create succeeds");

        assert_eq!(inserts(&log)[0].client_key, Some(supplied));
    }

    #[tokio::test]
    async fn omitted_status_defaults_to_enabled() {
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();
        req.status = None;

        use_case(&log, false)
            .create_authority(&mut req)
            .await
            .expect("create succeeds");

        assert_eq!(
            inserts(&log)[0]
                .status
                .as_deref(),
            Some("enabled")
        );
    }

    #[tokio::test]
    async fn supplied_status_is_kept() {
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();
        req.status = Some(AuthorityStatus::Disabled);

        use_case(&log, false)
            .create_authority(&mut req)
            .await
            .expect("create succeeds");

        let captured = inserts(&log);
        assert_eq!(captured[0].status.as_deref(), Some("disabled"));
        assert_eq!(captured[0].name, "new-authority");
        assert_eq!(captured[0].strategy, "username_password");
        assert_eq!(captured[0].params, json!({ "password_salt": "pepper" }));
    }

    #[tokio::test]
    async fn insert_failure_propagates() {
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();

        let error = use_case(&log, true)
            .create_authority(&mut req)
            .await
            .expect_err("insert failure surfaces");

        assert_eq!(error.to_string(), "simulated insert failure");
        assert_eq!(inserts(&log).len(), 1, "the insert was attempted");
    }
}
