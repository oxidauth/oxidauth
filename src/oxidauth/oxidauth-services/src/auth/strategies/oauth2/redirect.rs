use async_trait::async_trait;
use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use oxidauth_kernel::{
    auth::oauth2::{
        redirect::{Oauth2RedirectParams, Oauth2RedirectResponse, Oauth2RedirectServiceTrait},
        state::CreateOAuthState,
    },
    authorities::{AuthorityNotFoundError, find_authority_by_client_key::*},
    error::BoxedError,
};
use oxidauth_repository::{
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    oauth_states::insert_oauth_state::InsertOAuthStateQuery,
};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};

use super::{AuthorityParams, OAuthFlavors};

/// A minted state lives for minutes — long enough to finish an IdP hop,
/// short enough that states captured from URLs/logs are inert by the time
/// anyone cares (OXA-000010).
pub const STATE_TTL_MINUTES: i64 = 10;

/// 128 bits of CSPRNG output — unguessable and unforgeable offline.
pub const STATE_NONCE_BYTES: usize = 16;

pub struct Oauth2RedirectUseCase<T, S>
where
    T: SelectAuthorityByClientKeyQuery,
    S: InsertOAuthStateQuery,
{
    authorities: T,
    oauth_states: S,
}

impl<T, S> Oauth2RedirectUseCase<T, S>
where
    T: SelectAuthorityByClientKeyQuery,
    S: InsertOAuthStateQuery,
{
    pub fn new(authorities: T, oauth_states: S) -> Self {
        Self {
            authorities,
            oauth_states,
        }
    }
}

#[async_trait]
impl<T, S> Oauth2RedirectServiceTrait for Oauth2RedirectUseCase<T, S>
where
    T: SelectAuthorityByClientKeyQuery,
    S: InsertOAuthStateQuery,
{
    #[tracing::instrument(name = "Oauth2RedirectUseCase::oauth2_redirect", skip(self))]
    async fn oauth2_redirect(
        &self,
        params: &Oauth2RedirectParams,
    ) -> Result<Oauth2RedirectResponse, BoxedError> {
        let authority = self
            .authorities
            .select_authority_by_client_key(&FindAuthorityByClientKey {
                client_key: params.client_key,
            })
            .await?
            .ok_or_else(|| AuthorityNotFoundError::client_key(params.client_key))?;

        let oauth_params: AuthorityParams = authority.params.try_into()?;

        // OXA-000010: an opaque single-use nonce recorded server-side as
        // sha256(state) with a short TTL. The old state — an argon2 hash of
        // the PUBLIC client_key — was forgeable offline and replayable
        // forever; this one proves origin (only this server knows the raw
        // value until it hands it out), freshness (expires_at), and
        // single-use (the callback consumes it) — none of which required a
        // slow KDF, because entropy, not secrecy of a public value, is what
        // makes a nonce unguessable.
        let state = new_state();

        self.oauth_states
            .insert_oauth_state(&CreateOAuthState {
                state_hash: Sha256::digest(state.as_bytes()).to_vec(),
                client_key: authority.client_key,
                expires_at: Utc::now() + Duration::minutes(STATE_TTL_MINUTES),
            })
            .await?;

        let redirect_url = match oauth_params.flavor {
            OAuthFlavors::Google => {
                let mut redirect_url = oauth_params.redirect_url;

                let combined_redirect_url = redirect_url
                    .query_pairs_mut()
                    .append_pair("response_type", "code")
                    .append_pair("include_granted_scopes", "true")
                    .append_pair("state", state.as_str())
                    .finish();

                combined_redirect_url.to_owned()
            },
            OAuthFlavors::Microsoft => {
                let mut redirect_url = oauth_params.redirect_url;

                let combined_redirect_url = redirect_url
                    .query_pairs_mut()
                    .append_pair("response_type", "code")
                    .append_pair("response_mode", "query")
                    .append_pair("state", state.as_str())
                    .finish();

                if let Some(email) = &params.email {
                    combined_redirect_url
                        .query_pairs_mut()
                        .append_pair("login_hint", email)
                        .finish();
                };

                combined_redirect_url.to_owned()
            },
        };

        Ok(Oauth2RedirectResponse { redirect_url })
    }
}

/// Mints a fresh state nonce: 128 bits from the CSPRNG, base64url (unpadded)
/// so it survives a round-trip through IdP query strings untouched.
pub fn new_state() -> String {
    let mut bytes = [0u8; STATE_NONCE_BYTES];
    OsRng.fill_bytes(&mut bytes);

    BASE64_URL_SAFE_NO_PAD.encode(bytes)
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
        auth::oauth2::state::CreateOAuthState,
        authorities::{
            Authority,
            AuthoritySettings,
            AuthorityStatus,
            AuthorityStrategy,
            TotpSettings,
            find_authority_by_client_key::FindAuthorityByClientKey,
        },
        jwt::EntitlementsEncoding,
    };
    use uuid::Uuid;

    use super::*;
    use crate::auth::strategies::oauth2::fixtures::{CLIENT_KEY, authority_params};

    struct MockAuthorityRepo {
        found: bool,
        broken_params: bool,
        flavor: OAuthFlavors,
        log: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl SelectAuthorityByClientKeyQuery for MockAuthorityRepo {
        async fn select_authority_by_client_key(
            &self,
            req: &FindAuthorityByClientKey,
        ) -> Result<Option<Authority>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .push(format!("find_authority:{}", req.client_key));

            if !self.found {
                return Ok(None);
            }

            let params = if self.broken_params {
                JsonValue::new(serde_json::json!({}))
            } else {
                authority_params(self.flavor)
                    .as_json_value()
                    .expect("fixture params serialize")
            };

            Ok(Some(Authority {
                id: Uuid::new_v4(),
                name: "oauth2-authority".to_owned(),
                client_key: CLIENT_KEY,
                status: AuthorityStatus::Enabled,
                strategy: AuthorityStrategy::Oauth2,
                settings: AuthoritySettings {
                    jwt_ttl: Duration::from_secs(900),
                    jwt_nbf_offset: Default::default(),
                    refresh_token_ttl: Duration::from_secs(86_400),
                    totp: TotpSettings::Disabled,
                    entitlements_encoding: EntitlementsEncoding::Txt,
                },
                params,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        }
    }

    struct MockStateStore {
        fail: bool,
        log: Arc<Mutex<Vec<String>>>,
        inserts: Arc<Mutex<Vec<(Vec<u8>, Uuid, i64)>>>,
    }

    #[async_trait]
    impl InsertOAuthStateQuery for MockStateStore {
        async fn insert_oauth_state(&self, params: &CreateOAuthState) -> Result<(), BoxedError> {
            if self.fail {
                return Err("simulated state store failure".into());
            }

            self.log
                .lock()
                .expect("log")
                .push(format!("insert_state:{}", params.client_key));
            self.inserts
                .lock()
                .expect("inserts")
                .push((
                    params.state_hash.clone(),
                    params.client_key,
                    params.expires_at.timestamp(),
                ));

            Ok(())
        }
    }

    fn use_case(
        found: bool,
        flavor: OAuthFlavors,
        broken_params: bool,
        state_fails: bool,
    ) -> (
        Oauth2RedirectUseCase<MockAuthorityRepo, MockStateStore>,
        Arc<Mutex<Vec<String>>>,
        Arc<Mutex<Vec<(Vec<u8>, Uuid, i64)>>>,
    ) {
        let log = Arc::new(Mutex::new(Vec::new()));
        let inserts = Arc::new(Mutex::new(Vec::new()));

        let case = Oauth2RedirectUseCase::new(
            MockAuthorityRepo {
                found,
                broken_params,
                flavor,
                log: log.clone(),
            },
            MockStateStore {
                fail: state_fails,
                log: log.clone(),
                inserts: inserts.clone(),
            },
        );

        (case, log, inserts)
    }

    fn ops(log: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
        log.lock()
            .expect("log")
            .clone()
    }

    fn query_pairs(url: &url::Url) -> Vec<(String, String)> {
        url.query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect()
    }

    fn state_of(pairs: &[(String, String)]) -> String {
        pairs
            .iter()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.clone())
            .expect("state pair present")
    }

    fn assert_state_shape(state: &str) {
        assert!(
            !state.starts_with("$argon2") && !state.contains('$'),
            "the state must be an opaque nonce, not a PHC string: {state}"
        );
        assert!(
            state.len() >= 16
                && state
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "the state must be url-safe base64 (got {state})"
        );
    }

    /// The mint must record sha256(state) bound to this client_key with a
    /// short TTL — the exact triple the callback consumes atomically.
    fn assert_store_row(
        inserts: &Arc<Mutex<Vec<(Vec<u8>, Uuid, i64)>>>,
        index: usize,
        state: &str,
    ) {
        let rows = inserts
            .lock()
            .expect("inserts");
        let (hash, client_key, expires_at) = &rows[index];

        assert_eq!(
            hash,
            &Sha256::digest(state.as_bytes()).to_vec(),
            "the stored digest must be sha256 of the raw state handed to the IdP"
        );
        assert_eq!(
            *client_key, CLIENT_KEY,
            "the state is bound to its authority"
        );

        let now = Utc::now().timestamp();
        assert!(
            (expires_at - (now + STATE_TTL_MINUTES * 60)).abs() <= 2,
            "the state expires roughly {STATE_TTL_MINUTES} minutes out, got {expires_at} vs {now}"
        );
    }

    #[tokio::test]
    async fn google_redirect_appends_response_type_scopes_and_state() {
        let (case, log, inserts) = use_case(true, OAuthFlavors::Google, false, false);

        let res = case
            .oauth2_redirect(&Oauth2RedirectParams {
                client_key: CLIENT_KEY,
                email: None,
            })
            .await
            .expect("google redirect builds");

        assert_eq!(
            res.redirect_url
                .as_str()
                .split_once('?')
                .expect("has query")
                .0,
            "https://authorize.example.com/oauth",
            "the stored authorize endpoint must be preserved"
        );

        let pairs = query_pairs(&res.redirect_url);
        let state = state_of(&pairs);

        // BUG(pinned): audit B2.1 expected client_id/scope/redirect_uri to be
        // appended here, but the implementation only appends
        // response_type/include_granted_scopes/state — those identity pairs
        // must be baked into the stored redirect_url and are carried over.
        assert_eq!(
            pairs
                .iter()
                .map(|(k, _)| k.as_str())
                .collect::<Vec<_>>(),
            vec![
                "client_id",
                "scope",
                "redirect_uri",
                "response_type",
                "include_granted_scopes",
                "state",
            ],
            "exact query contract: baked pairs preserved, three appended"
        );
        assert_eq!(pairs[3], ("response_type".to_owned(), "code".to_owned()));
        assert_eq!(
            pairs[4],
            ("include_granted_scopes".to_owned(), "true".to_owned())
        );

        // OXA-000010 (flipped pin): the state is an opaque CSPRNG nonce, not
        // an argon2 hash of anything
        assert_state_shape(&state);
        assert_store_row(&inserts, 0, &state);
        assert_eq!(
            ops(&log),
            vec![
                format!("find_authority:{CLIENT_KEY}"),
                format!("insert_state:{CLIENT_KEY}")
            ]
        );
    }

    #[tokio::test]
    async fn microsoft_redirect_appends_response_mode_and_state() {
        let (case, _, inserts) = use_case(true, OAuthFlavors::Microsoft, false, false);

        let res = case
            .oauth2_redirect(&Oauth2RedirectParams {
                client_key: CLIENT_KEY,
                email: None,
            })
            .await
            .expect("microsoft redirect builds");

        let pairs = query_pairs(&res.redirect_url);
        assert!(pairs.contains(&("response_type".to_owned(), "code".to_owned())));
        assert!(pairs.contains(&("response_mode".to_owned(), "query".to_owned())));
        assert!(
            !pairs
                .iter()
                .any(|(k, _)| k == "include_granted_scopes"),
            "google-only scope flag must not leak into microsoft urls"
        );
        assert!(
            !pairs
                .iter()
                .any(|(k, _)| k == "login_hint"),
            "no email means no login_hint"
        );

        let state = state_of(&pairs);
        assert_state_shape(&state);
        assert_store_row(&inserts, 0, &state);
    }

    #[tokio::test]
    async fn microsoft_redirect_adds_login_hint_for_a_given_email() {
        let (case, _, _) = use_case(true, OAuthFlavors::Microsoft, false, false);

        let res = case
            .oauth2_redirect(&Oauth2RedirectParams {
                client_key: CLIENT_KEY,
                email: Some("user@corp.example".to_owned()),
            })
            .await
            .expect("microsoft redirect with email builds");

        let pairs = query_pairs(&res.redirect_url);
        assert!(
            pairs.contains(&("login_hint".to_owned(), "user@corp.example".to_owned())),
            "login_hint must carry the email verbatim, got {pairs:?}"
        );
    }

    #[tokio::test]
    async fn every_mint_is_a_unique_nonce_bound_to_a_fresh_store_row() {
        let (case, _, inserts) = use_case(true, OAuthFlavors::Google, false, false);

        let first = case
            .oauth2_redirect(&Oauth2RedirectParams {
                client_key: CLIENT_KEY,
                email: None,
            })
            .await
            .expect("first redirect builds");
        let second = case
            .oauth2_redirect(&Oauth2RedirectParams {
                client_key: CLIENT_KEY,
                email: None,
            })
            .await
            .expect("second redirect builds");

        let first_state = state_of(&query_pairs(&first.redirect_url));
        let second_state = state_of(&query_pairs(&second.redirect_url));

        assert_ne!(
            first_state, second_state,
            "each redirect mints a fresh nonce"
        );
        assert_eq!(
            inserts
                .lock()
                .expect("inserts")
                .len(),
            2
        );
        assert_store_row(&inserts, 0, &first_state);
        assert_store_row(&inserts, 1, &second_state);
    }

    #[tokio::test]
    async fn unknown_client_key_reports_authority_not_found_without_minting() {
        let (case, log, inserts) = use_case(false, OAuthFlavors::Google, false, false);

        let err = case
            .oauth2_redirect(&Oauth2RedirectParams {
                client_key: CLIENT_KEY,
                email: None,
            })
            .await
            .expect_err("unknown client key must error");

        assert!(
            err.to_string()
                .contains(&format!("authority not found by client_key: {CLIENT_KEY}")),
            "expected AuthorityNotFoundError for the client key, got: {err}"
        );
        assert_eq!(ops(&log), vec![format!("find_authority:{CLIENT_KEY}")]);
        assert!(
            inserts
                .lock()
                .expect("inserts")
                .is_empty(),
            "no state may be minted for an unknown authority"
        );
    }

    #[tokio::test]
    async fn malformed_authority_params_error() {
        let (case, log, inserts) = use_case(true, OAuthFlavors::Google, true, false);

        let err = case
            .oauth2_redirect(&Oauth2RedirectParams {
                client_key: CLIENT_KEY,
                email: None,
            })
            .await
            .expect_err("authority without oauth params must error");
        assert!(
            err.to_string()
                .contains("missing field `exchange_url`"),
            "expected serde missing-field error, got: {err}"
        );
        assert!(
            inserts
                .lock()
                .expect("inserts")
                .is_empty(),
            "no state may be minted when the params will not load: {:?}",
            ops(&log)
        );
    }

    #[tokio::test]
    async fn state_store_failure_aborts_the_redirect() {
        let (case, _, _) = use_case(true, OAuthFlavors::Google, false, true);

        let err = case
            .oauth2_redirect(&Oauth2RedirectParams {
                client_key: CLIENT_KEY,
                email: None,
            })
            .await
            .expect_err("a redirect whose state cannot be recorded must not be issued");

        assert!(
            err.to_string()
                .contains("simulated state store failure"),
            "got: {err}"
        );
    }

    #[test]
    fn new_state_is_a_unique_url_safe_csprng_nonce() {
        let first = new_state();
        let second = new_state();

        assert_ne!(first, second, "each state nonce must be fresh");
        assert_state_shape(&first);

        let decoded = BASE64_URL_SAFE_NO_PAD
            .decode(&first)
            .expect("base64url decodes");
        assert_eq!(
            decoded.len(),
            STATE_NONCE_BYTES,
            "the nonce carries STATE_NONCE_BYTES of CSPRNG output"
        );
    }
}
