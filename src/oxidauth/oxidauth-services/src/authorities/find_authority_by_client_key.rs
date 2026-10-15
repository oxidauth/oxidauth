use async_trait::async_trait;
use oxidauth_kernel::{
    authorities::{AuthorityNotFoundError, find_authority_by_client_key::*},
    error::BoxedError,
};
use oxidauth_repository::authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery;

pub struct FindAuthorityByClientKeyUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
{
    authorities: T,
}

impl<T> FindAuthorityByClientKeyUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<T> FindAuthorityByClientKeyServiceTrait for FindAuthorityByClientKeyUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
{
    #[tracing::instrument(
        name = "FindAuthorityByClientKeyUseCase::find_authority_by_client_key",
        skip(self)
    )]
    async fn find_authority_by_client_key(
        &self,
        params: &FindAuthorityByClientKey,
    ) -> Result<Authority, BoxedError> {
        let authority = self
            .authorities
            .select_authority_by_client_key(params)
            .await?
            .ok_or_else(|| AuthorityNotFoundError::client_key(params.client_key))?;

        Ok(authority)
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
        authorities::{AuthoritySettings, AuthorityStatus, AuthorityStrategy, TotpSettings},
        jwt::EntitlementsEncoding,
    };
    use uuid::Uuid;

    use super::*;

    #[derive(Clone)]
    struct MockAuthorityRepository {
        found: bool,
        log: Arc<Mutex<Vec<String>>>,
    }

    impl MockAuthorityRepository {
        fn new(found: bool) -> Self {
            Self {
                found,
                log: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    #[async_trait]
    impl SelectAuthorityByClientKeyQuery for MockAuthorityRepository {
        async fn select_authority_by_client_key(
            &self,
            params: &FindAuthorityByClientKey,
        ) -> Result<Option<Authority>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .push(format!("select_authority:{}", params.client_key));

            if !self.found {
                return Ok(None);
            }

            Ok(Some(Authority {
                id: Uuid::new_v4(),
                name: "keyed-authority".to_owned(),
                client_key: params.client_key,
                status: AuthorityStatus::Enabled,
                strategy: AuthorityStrategy::UsernamePassword,
                settings: AuthoritySettings {
                    jwt_ttl: Duration::from_secs(900),
                    jwt_nbf_offset: Default::default(),
                    refresh_token_ttl: Duration::from_secs(86_400),
                    totp: TotpSettings::Disabled,
                    entitlements_encoding: EntitlementsEncoding::Txt,
                },
                params: JsonValue::new(serde_json::json!({})),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        }
    }

    fn use_case(
        found: bool,
    ) -> (
        FindAuthorityByClientKeyUseCase<MockAuthorityRepository>,
        Arc<Mutex<Vec<String>>>,
    ) {
        let repo = MockAuthorityRepository::new(found);
        let log = repo.log.clone();

        (FindAuthorityByClientKeyUseCase::new(repo), log)
    }

    fn params() -> FindAuthorityByClientKey {
        FindAuthorityByClientKey {
            client_key: Uuid::new_v4(),
        }
    }

    /// The key is the whole answer: the row that comes back is the one that
    /// carries it, not merely a row that exists.
    #[tokio::test]
    async fn a_matching_key_returns_that_authority() {
        let (case, log) = use_case(true);
        let params = params();

        let authority = case
            .find_authority_by_client_key(&params)
            .await
            .expect("the keyed authority is returned");

        assert_eq!(authority.client_key, params.client_key);

        let logged = log
            .lock()
            .expect("log")
            .clone();
        assert_eq!(
            logged,
            vec![format!("select_authority:{}", params.client_key)]
        );
    }

    /// A miss is an error naming the key, not `Ok(None)`: callers branch on the
    /// error (bootstrap creates the system authority on that branch), and a
    /// silent `None` would hand an absent authority to whoever asked.
    #[tokio::test]
    async fn a_missing_key_is_authority_not_found_naming_the_key() {
        let (case, _log) = use_case(false);
        let params = params();

        let err = case
            .find_authority_by_client_key(&params)
            .await
            .expect_err("an unknown key must not resolve to an authority");

        assert!(
            err.to_string()
                .contains(&params.client_key.to_string()),
            "the error must name the key that missed: {err}"
        );
    }
}
