use std::fmt::Error;

use argon2::{
    Argon2,
    password_hash::{Error as HashError, PasswordHasher, SaltString},
};
use async_trait::async_trait;
use oxidauth_kernel::{
    auth::oauth2::redirect::{
        Oauth2RedirectParams,
        Oauth2RedirectResponse,
        Oauth2RedirectServiceTrait,
    },
    authorities::{AuthorityNotFoundError, find_authority_by_client_key::*},
    error::BoxedError,
};
use oxidauth_repository::authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery;
use rand_core::OsRng;
use uuid::Uuid;

use super::{AuthorityParams, OAuthFlavors};

pub struct Oauth2RedirectUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
{
    authorities: T,
}

impl<T> Oauth2RedirectUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<T> Oauth2RedirectServiceTrait for Oauth2RedirectUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
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

        let Ok(state_hash) = hash_client_id(authority.client_key) else {
            let err = Error;
            return Err(Box::new(err));
        };

        let redirect_url = match oauth_params.flavor {
            OAuthFlavors::Google => {
                let mut redirect_url = oauth_params.redirect_url;

                let combined_redirect_url = redirect_url
                    .query_pairs_mut()
                    .append_pair("response_type", "code")
                    .append_pair("include_granted_scopes", "true")
                    .append_pair("state", state_hash.as_str())
                    .finish();

                combined_redirect_url.to_owned()
            },
            OAuthFlavors::Microsoft => {
                let mut redirect_url = oauth_params.redirect_url;

                let combined_redirect_url = redirect_url
                    .query_pairs_mut()
                    .append_pair("response_type", "code")
                    .append_pair("response_mode", "query")
                    .append_pair("state", state_hash.as_str())
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

pub fn hash_client_id(client_id: Uuid) -> Result<String, HashError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(&client_id.into_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use argon2::{PasswordHash, PasswordVerifier};
    use chrono::Utc;
    use oxidauth_kernel::{
        authorities::{Authority, AuthoritySettings, AuthorityStrategy, TotpSettings},
        jwt::EntitlementsEncoding,
    };
    use uuid::Uuid;

    use super::*;
    use crate::auth::strategies::oauth2::fixtures::{CLIENT_KEY, authority_params};

    struct MockAuthorityRepo {
        found: bool,
        broken_params: bool,
        flavor: OAuthFlavors,
        requests: Arc<Mutex<Vec<Uuid>>>,
    }

    #[async_trait]
    impl SelectAuthorityByClientKeyQuery for MockAuthorityRepo {
        async fn select_authority_by_client_key(
            &self,
            req: &FindAuthorityByClientKey,
        ) -> Result<Option<Authority>, BoxedError> {
            self.requests
                .lock()
                .expect("log")
                .push(req.client_key);

            if !self.found {
                return Ok(None);
            }

            let params = if self.broken_params {
                oxidauth_kernel::JsonValue::new(serde_json::json!({}))
            } else {
                authority_params(self.flavor)
                    .as_json_value()
                    .expect("fixture params serialize")
            };

            Ok(Some(Authority {
                id: Uuid::new_v4(),
                name: "oauth2-authority".to_owned(),
                client_key: CLIENT_KEY,
                status: oxidauth_kernel::authorities::AuthorityStatus::Enabled,
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

    fn use_case(
        found: bool,
        flavor: OAuthFlavors,
        broken_params: bool,
    ) -> (
        Oauth2RedirectUseCase<MockAuthorityRepo>,
        Arc<Mutex<Vec<Uuid>>>,
    ) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let mock = MockAuthorityRepo {
            found,
            broken_params,
            flavor,
            requests: requests.clone(),
        };

        (Oauth2RedirectUseCase::new(mock), requests)
    }

    fn query_pairs(url: &url::Url) -> Vec<(String, String)> {
        url.query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect()
    }

    fn assert_state_verifies_client_key(state: &str, client_key: Uuid) {
        let hash = PasswordHash::new(state).expect("state is a PHC hash");

        assert!(
            Argon2::default()
                .verify_password(&client_key.into_bytes(), &hash)
                .is_ok(),
            "state must be an argon2 hash of the authority client key"
        );
    }

    #[tokio::test]
    async fn google_redirect_appends_response_type_scopes_and_state() {
        let (case, requests) = use_case(true, OAuthFlavors::Google, false);

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
        let state = pairs
            .iter()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.clone())
            .expect("state pair present");

        // BUG(pinned): audit B2.1 expected client_id/scope/redirect_uri to be
        // appended here, but the implementation only appends
        // response_type/include_granted_scopes/state — those identity pairs
        // must be baked into the stored redirect_url and are carried over.
        assert_eq!(
            pairs,
            vec![
                ("client_id".to_owned(), "sso-client-id".to_owned()),
                ("scope".to_owned(), "openid email".to_owned()),
                (
                    "redirect_uri".to_owned(),
                    "https://app.example.com/cb".to_owned()
                ),
                ("response_type".to_owned(), "code".to_owned()),
                ("include_granted_scopes".to_owned(), "true".to_owned()),
                ("state".to_owned(), state.clone()),
            ],
            "exact query contract: baked pairs preserved, three appended"
        );

        assert!(state.starts_with("$argon2"));
        assert_state_verifies_client_key(&state, CLIENT_KEY);
        assert_eq!(
            requests
                .lock()
                .expect("log")
                .as_slice(),
            &[CLIENT_KEY]
        );
    }

    #[tokio::test]
    async fn microsoft_redirect_appends_response_mode_and_state() {
        let (case, _) = use_case(true, OAuthFlavors::Microsoft, false);

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

        let state = pairs
            .iter()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.clone())
            .expect("state pair present");
        assert_state_verifies_client_key(&state, CLIENT_KEY);
    }

    #[tokio::test]
    async fn microsoft_redirect_adds_login_hint_for_a_given_email() {
        let (case, _) = use_case(true, OAuthFlavors::Microsoft, false);

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
    async fn unknown_client_key_reports_authority_not_found() {
        let (case, requests) = use_case(false, OAuthFlavors::Google, false);

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
        assert_eq!(
            requests
                .lock()
                .expect("log")
                .as_slice(),
            &[CLIENT_KEY]
        );
    }

    #[tokio::test]
    async fn malformed_authority_params_error() {
        let (case, _) = use_case(true, OAuthFlavors::Google, true);

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
    }

    #[test]
    fn hash_client_id_is_a_verifiable_random_salted_hash() {
        let client_id = CLIENT_KEY;

        let first = hash_client_id(client_id).expect("hashing succeeds");
        let second = hash_client_id(client_id).expect("hashing succeeds");

        assert!(first.starts_with("$argon2"));
        assert_ne!(first, second, "each state hash must use a fresh salt");
        assert_state_verifies_client_key(&first, client_id);
        assert_state_verifies_client_key(&second, client_id);

        // a different client key must not verify against this state hash
        let other = Uuid::new_v4();
        let hash = PasswordHash::new(&first).expect("PHC parse");
        assert!(
            Argon2::default()
                .verify_password(&other.into_bytes(), &hash)
                .is_err(),
            "state must be bound to the specific client key"
        );
    }
}
