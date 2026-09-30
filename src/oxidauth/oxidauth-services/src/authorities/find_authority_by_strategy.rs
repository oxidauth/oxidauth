use async_trait::async_trait;
use oxidauth_kernel::{
    authorities::{AuthorityNotFoundError, find_authority_by_strategy::*},
    error::BoxedError,
};
use oxidauth_repository::authorities::select_authority_by_strategy::SelectAuthorityByStrategyQuery;

pub struct FindAuthorityByStrategyUseCase<T>
where
    T: SelectAuthorityByStrategyQuery,
{
    authorities: T,
}

impl<T> FindAuthorityByStrategyUseCase<T>
where
    T: SelectAuthorityByStrategyQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<T> FindAuthorityByStrategyServiceTrait for FindAuthorityByStrategyUseCase<T>
where
    T: SelectAuthorityByStrategyQuery,
{
    #[tracing::instrument(
        name = "FindAuthorityByStrategyUseCase::find_authority_by_strategy",
        skip(self)
    )]
    async fn find_authority_by_strategy(
        &self,
        params: &FindAuthorityByStrategy,
    ) -> Result<Authority, BoxedError> {
        let authority = self
            .authorities
            .select_authority_by_strategy(params)
            .await?
            .ok_or_else(|| AuthorityNotFoundError::strategy(params.strategy))?;

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
        authorities::{AuthoritySettings, AuthorityStatus, TotpSettings},
        jwt::EntitlementsEncoding,
    };

    use super::*;

    fn authority() -> Authority {
        Authority {
            id: uuid::uuid!("66666666-6666-4666-8666-666666666666"),
            name: "oauth2-authority".to_owned(),
            client_key: uuid::Uuid::new_v4(),
            status: AuthorityStatus::Enabled,
            strategy: AuthorityStrategy::Oauth2,
            settings: AuthoritySettings {
                jwt_ttl: Duration::from_secs(900),
                jwt_nbf_offset: Default::default(),
                refresh_token_ttl: Duration::from_secs(86_400),
                totp: TotpSettings::Disabled,
                entitlements_encoding: EntitlementsEncoding::Txt,
            },
            params: JsonValue::empty(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Default)]
    struct Log {
        finds: Vec<String>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn finds(log: &SharedLog) -> Vec<String> {
        log.lock()
            .expect("log")
            .finds
            .clone()
    }

    struct MockSelectByStrategy {
        log: SharedLog,
        outcome: Outcome,
    }

    enum Outcome {
        Found,
        Empty,
        Fail,
    }

    #[async_trait]
    impl SelectAuthorityByStrategyQuery for MockSelectByStrategy {
        async fn select_authority_by_strategy(
            &self,
            req: &FindAuthorityByStrategy,
        ) -> Result<Option<Authority>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .finds
                .push(req.strategy.to_string());

            match self.outcome {
                Outcome::Found => Ok(Some(authority())),
                Outcome::Empty => Ok(None),
                Outcome::Fail => Err("simulated database failure".into()),
            }
        }
    }

    fn use_case(
        log: &SharedLog,
        outcome: Outcome,
    ) -> FindAuthorityByStrategyUseCase<MockSelectByStrategy> {
        FindAuthorityByStrategyUseCase::new(MockSelectByStrategy {
            log: log.clone(),
            outcome,
        })
    }

    #[tokio::test]
    async fn matching_authority_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));
        let params = FindAuthorityByStrategy {
            strategy: AuthorityStrategy::Oauth2,
        };

        let found = use_case(&log, Outcome::Found)
            .find_authority_by_strategy(&params)
            .await
            .expect("strategy found");

        assert_eq!(found.name, "oauth2-authority");
        assert_eq!(found.strategy.to_string(), "oauth2");
        assert_eq!(finds(&log), vec!["oauth2".to_owned()]);
    }

    #[tokio::test]
    async fn empty_lookup_maps_to_authority_not_found_by_strategy() {
        let log = Arc::new(Mutex::new(Log::default()));
        let params = FindAuthorityByStrategy {
            strategy: AuthorityStrategy::UsernamePassword,
        };

        // This is the only authorities use case that maps "no row" to a
        // domain error: AuthorityNotFoundError::Strategy.
        let error = use_case(&log, Outcome::Empty)
            .find_authority_by_strategy(&params)
            .await
            .expect_err("no row must fail");

        assert_eq!(
            format!("{error:?}"),
            "Strategy(UsernamePassword)",
            "the boxed error carries the AuthorityNotFoundError::Strategy \
             variant (this Debug string is what the api/hurl layer asserts)"
        );
        assert_eq!(
            error.to_string(),
            "authority not found by strategy: username_password"
        );
        assert_eq!(finds(&log), vec!["username_password".to_owned()]);
    }

    #[tokio::test]
    async fn database_failure_propagates_without_not_found_mapping() {
        let log = Arc::new(Mutex::new(Log::default()));
        let params = FindAuthorityByStrategy {
            strategy: AuthorityStrategy::Oauth2,
        };

        let error = use_case(&log, Outcome::Fail)
            .find_authority_by_strategy(&params)
            .await
            .expect_err("database failure surfaces");

        assert_eq!(error.to_string(), "simulated database failure");
        assert!(!(error.to_string()).contains("authority not found"));
    }
}
