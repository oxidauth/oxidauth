use async_trait::async_trait;
use oxidauth_kernel::{
    authorities::{TotpSettings, update_authority::*},
    error::BoxedError,
    totp_secrets::create_totp_secrets_by_authority_id::{
        CreateTotpSecrets,
        CreateTotpSecretsService,
        CreateTotpSecretsTrait,
    },
};
use oxidauth_repository::authorities::{
    select_authority_by_id::*,
    update_authority::UpdateAuthorityQuery,
};
use uuid::Uuid;

pub struct UpdateAuthorityUseCase<T, I>
where
    T: UpdateAuthorityQuery,
    I: SelectAuthorityByIdQuery,
{
    update_authority: T,
    authority_by_id: I,
    totp_secrets: CreateTotpSecretsService,
}

impl<T, I> UpdateAuthorityUseCase<T, I>
where
    T: UpdateAuthorityQuery,
    I: SelectAuthorityByIdQuery,
{
    pub fn new(
        update_authority: T,
        authority_by_id: I,
        totp_secrets: CreateTotpSecretsService,
    ) -> Self {
        Self {
            update_authority,
            authority_by_id,
            totp_secrets,
        }
    }
}

#[async_trait]
impl<T, I> UpdateAuthorityServiceTrait for UpdateAuthorityUseCase<T, I>
where
    T: UpdateAuthorityQuery,
    I: SelectAuthorityByIdQuery,
{
    #[tracing::instrument(name = "UpdateAuthorityUseCase::update_authority", skip(self))]
    async fn update_authority(&self, req: &mut UpdateAuthority) -> Result<Authority, BoxedError> {
        let authority_id = req
            .id
            .ok_or("UpdateAuthority must have authority_id")?;

        let current = self
            .authority_by_id
            .select_authority_by_id(&FindAuthorityById { authority_id })
            .await?;

        if req.client_key.is_none() {
            req.client_key
                .replace(Uuid::new_v4());
        }

        if req.status.is_none() {
            req.status
                .replace(current.status);
        }

        if let (TotpSettings::Disabled, TotpSettings::Enabled { .. }) =
            (&current.settings.totp, &req.settings.totp)
        {
            self.totp_secrets
                .create_totp_secrets(&CreateTotpSecrets { authority_id })
                .await?;
        }

        self.update_authority
            .update_authority(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use oxidauth_kernel::{JsonValue, jwt::EntitlementsEncoding};
    use serde_json::json;
    use url::Url;

    use super::*;

    fn authority_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    /// The client_key stored on the *current* (pre-update) authority row.
    fn current_client_key() -> Uuid {
        uuid::uuid!("22222222-2222-4222-8222-222222222222")
    }

    fn settings(totp: TotpSettings) -> AuthoritySettings {
        AuthoritySettings {
            jwt_ttl: Duration::from_secs(900),
            jwt_nbf_offset: Default::default(),
            refresh_token_ttl: Duration::from_secs(86_400),
            totp,
            entitlements_encoding: EntitlementsEncoding::Txt,
        }
    }

    fn totp_enabled() -> TotpSettings {
        TotpSettings::Enabled {
            totp_ttl: Duration::from_secs(30),
            webhook: Url::parse("https://example.com/totp-hook").expect("url"),
            webhook_key: "hook-key".to_owned(),
        }
    }

    fn current_authority(status: AuthorityStatus, totp: TotpSettings) -> Authority {
        Authority {
            id: authority_id(),
            name: "update-authority".to_owned(),
            client_key: current_client_key(),
            status,
            strategy: AuthorityStrategy::UsernamePassword,
            settings: settings(totp),
            params: JsonValue::empty(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Snapshot of what the use case finally hands to the update query.
    #[derive(Clone, Debug, PartialEq)]
    struct CapturedUpdate {
        id: Option<Uuid>,
        name: String,
        client_key: Option<Uuid>,
        status: Option<String>,
        totp_enabled: bool,
        params: serde_json::Value,
    }

    #[derive(Default)]
    struct Log {
        find_by_id: Vec<Uuid>,
        updates: Vec<CapturedUpdate>,
        totp_secret_batches: Vec<Uuid>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn find_by_id_calls(log: &SharedLog) -> Vec<Uuid> {
        log.lock()
            .expect("log")
            .find_by_id
            .clone()
    }

    fn updates(log: &SharedLog) -> Vec<CapturedUpdate> {
        log.lock()
            .expect("log")
            .updates
            .clone()
    }

    fn totp_batches(log: &SharedLog) -> Vec<Uuid> {
        log.lock()
            .expect("log")
            .totp_secret_batches
            .clone()
    }

    struct MockFindById {
        log: SharedLog,
        current_status_disabled: bool,
        current_totp_enabled: bool,
        fail: bool,
    }

    #[async_trait]
    impl SelectAuthorityByIdQuery for MockFindById {
        async fn select_authority_by_id(
            &self,
            req: &FindAuthorityById,
        ) -> Result<Authority, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .find_by_id
                .push(req.authority_id);

            if self.fail {
                return Err("simulated authority lookup failure".into());
            }

            Ok(current_authority(
                if self.current_status_disabled {
                    AuthorityStatus::Disabled
                } else {
                    AuthorityStatus::Enabled
                },
                if self.current_totp_enabled {
                    totp_enabled()
                } else {
                    TotpSettings::Disabled
                },
            ))
        }
    }

    struct MockUpdate {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl UpdateAuthorityQuery for MockUpdate {
        async fn update_authority(&self, req: &UpdateAuthority) -> Result<Authority, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .updates
                .push(CapturedUpdate {
                    id: req.id,
                    name: req.name.clone(),
                    client_key: req.client_key,
                    status: req
                        .status
                        .as_ref()
                        .map(|status| status.to_string()),
                    totp_enabled: matches!(&req.settings.totp, TotpSettings::Enabled { .. }),
                    params: req.params.clone(),
                });

            if self.fail {
                return Err("simulated update failure".into());
            }

            Ok(current_authority(
                AuthorityStatus::Enabled,
                TotpSettings::Disabled,
            ))
        }
    }

    struct MockTotpSecrets {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl CreateTotpSecretsTrait for MockTotpSecrets {
        async fn create_totp_secrets(&self, req: &CreateTotpSecrets) -> Result<(), BoxedError> {
            self.log
                .lock()
                .expect("log")
                .totp_secret_batches
                .push(req.authority_id);

            if self.fail {
                return Err("simulated totp secret creation failure".into());
            }

            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn use_case(
        log: &SharedLog,
        current_status_disabled: bool,
        current_totp_enabled: bool,
        find_fails: bool,
        update_fails: bool,
        totp_fails: bool,
    ) -> UpdateAuthorityUseCase<MockUpdate, MockFindById> {
        UpdateAuthorityUseCase::new(
            MockUpdate {
                log: log.clone(),
                fail: update_fails,
            },
            MockFindById {
                log: log.clone(),
                current_status_disabled,
                current_totp_enabled,
                fail: find_fails,
            },
            Arc::new(MockTotpSecrets {
                log: log.clone(),
                fail: totp_fails,
            }),
        )
    }

    fn base_request() -> UpdateAuthority {
        UpdateAuthority {
            id: Some(authority_id()),
            name: "renamed-authority".to_owned(),
            client_key: None,
            status: None,
            strategy: AuthorityStrategy::UsernamePassword,
            settings: settings(TotpSettings::Disabled),
            params: json!({ "password_salt": "pepper" }),
        }
    }

    #[tokio::test]
    async fn supplied_client_key_is_preserved_verbatim() {
        let log = Arc::new(Mutex::new(Log::default()));
        let replacement = uuid::Uuid::new_v4();
        let mut req = base_request();
        req.client_key = Some(replacement);

        use_case(&log, false, false, false, false, false)
            .update_authority(&mut req)
            .await
            .expect("update succeeds");

        let captured = updates(&log);
        assert_eq!(captured.len(), 1);
        assert_eq!(
            captured[0].client_key,
            Some(replacement),
            "a supplied client_key must reach the repository untouched"
        );
        assert_eq!(captured[0].id, Some(authority_id()));
        assert_eq!(captured[0].name, "renamed-authority");
        assert_eq!(captured[0].params, json!({ "password_salt": "pepper" }));
        assert!(find_by_id_calls(&log).contains(&authority_id()));
    }

    #[tokio::test]
    async fn omitted_client_key_is_regenerated() {
        // Design-pin (OXA-000003, ruled BY DESIGN 2026-09-29 — wontfix): omitting client_key on
        // update regenerates it — `update_authority.rs:61-63` replaces a `None` client_key with a
        // fresh `Uuid::new_v4()`. Operators must re-publish the new key to dependent IdP consoles
        // in the same change; do not "fix" this to keep-on-omit. The hurl suite asserts the
        // behavior end-to-end.
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();
        req.client_key = None;

        use_case(&log, false, false, false, false, false)
            .update_authority(&mut req)
            .await
            .expect("update succeeds");

        let captured = updates(&log);
        assert_eq!(captured.len(), 1);

        let regenerated = captured[0]
            .client_key
            .expect("a new client_key was generated");

        assert_ne!(
            regenerated,
            current_client_key(),
            "the regenerated key silently replaces the row's existing key"
        );
        assert_eq!(
            regenerated.get_version(),
            Some(uuid::Version::Random),
            "the replacement is a freshly generated v4 Uuid"
        );
    }

    #[tokio::test]
    async fn every_omission_gets_a_fresh_client_key() {
        let log = Arc::new(Mutex::new(Log::default()));
        let case = use_case(&log, false, false, false, false, false);

        let mut first = base_request();
        let mut second = base_request();

        case.update_authority(&mut first)
            .await
            .expect("first update succeeds");
        case.update_authority(&mut second)
            .await
            .expect("second update succeeds");

        let captured = updates(&log);
        let keys: Vec<Uuid> = captured
            .iter()
            .map(|c| {
                c.client_key
                    .expect("generated")
            })
            .collect();

        assert_eq!(keys.len(), 2);
        assert_ne!(keys[0], keys[1], "each update mints a new key");
    }

    #[tokio::test]
    async fn missing_authority_id_fails_before_touching_the_database() {
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();
        req.id = None;

        let error = use_case(&log, false, false, false, false, false)
            .update_authority(&mut req)
            .await
            .expect_err("no id, no update");

        assert_eq!(error.to_string(), "UpdateAuthority must have authority_id");
        assert!(find_by_id_calls(&log).is_empty());
        assert!(updates(&log).is_empty());
    }

    #[tokio::test]
    async fn omitted_status_is_backfilled_from_the_current_authority() {
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();
        req.status = None;

        use_case(&log, true, false, false, false, false)
            .update_authority(&mut req)
            .await
            .expect("update succeeds");

        let captured = updates(&log);
        assert_eq!(
            captured[0].status.as_deref(),
            Some("disabled"),
            "a None status inherits the current row's status"
        );
    }

    #[tokio::test]
    async fn supplied_status_is_kept() {
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();
        req.status = Some(AuthorityStatus::Disabled);

        use_case(&log, false, false, false, false, false)
            .update_authority(&mut req)
            .await
            .expect("update succeeds");

        let captured = updates(&log);
        assert_eq!(captured[0].status.as_deref(), Some("disabled"));
    }

    #[tokio::test]
    async fn totp_disabled_to_enabled_mints_secrets_for_the_authority() {
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();
        req.settings = settings(totp_enabled());

        use_case(&log, false, false, false, false, false)
            .update_authority(&mut req)
            .await
            .expect("update succeeds");

        assert_eq!(totp_batches(&log), vec![authority_id()]);
        assert_eq!(updates(&log)[0].totp_enabled, true);
    }

    #[tokio::test]
    async fn totp_transitions_without_the_upward_edge_skip_secret_minting() {
        // disabled -> disabled: no edge crossed.
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();
        req.settings = settings(TotpSettings::Disabled);

        use_case(&log, false, false, false, false, false)
            .update_authority(&mut req)
            .await
            .expect("update succeeds");

        assert!(totp_batches(&log).is_empty());

        // enabled -> enabled: not a Disabled-to-Enabled transition.
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();
        req.settings = settings(totp_enabled());

        use_case(&log, false, true, false, false, false)
            .update_authority(&mut req)
            .await
            .expect("update succeeds");

        assert!(totp_batches(&log).is_empty());

        // enabled -> disabled: tearing TOTP down never mints secrets.
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();
        req.settings = settings(TotpSettings::Disabled);

        use_case(&log, false, true, false, false, false)
            .update_authority(&mut req)
            .await
            .expect("update succeeds");

        assert!(totp_batches(&log).is_empty());
    }

    #[tokio::test]
    async fn lookup_failure_propagates_and_updates_nothing() {
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();

        let error = use_case(&log, false, false, true, false, false)
            .update_authority(&mut req)
            .await
            .expect_err("lookup failure surfaces");

        assert_eq!(error.to_string(), "simulated authority lookup failure");
        assert!(updates(&log).is_empty());
    }

    #[tokio::test]
    async fn update_failure_propagates() {
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();

        let error = use_case(&log, false, false, false, true, false)
            .update_authority(&mut req)
            .await
            .expect_err("update failure surfaces");

        assert_eq!(error.to_string(), "simulated update failure");
        assert_eq!(updates(&log).len(), 1, "the write was attempted");
    }

    #[tokio::test]
    async fn totp_secret_minting_failure_aborts_before_the_update() {
        let log = Arc::new(Mutex::new(Log::default()));
        let mut req = base_request();
        req.settings = settings(totp_enabled());

        let error = use_case(&log, false, false, false, false, true)
            .update_authority(&mut req)
            .await
            .expect_err("totp failure surfaces");

        assert_eq!(error.to_string(), "simulated totp secret creation failure");
        assert_eq!(totp_batches(&log), vec![authority_id()]);
        assert!(
            updates(&log).is_empty(),
            "the authority row is never flipped to totp-enabled without secrets"
        );
    }
}
