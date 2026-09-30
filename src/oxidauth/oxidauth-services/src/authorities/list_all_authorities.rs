use async_trait::async_trait;
use oxidauth_kernel::{authorities::list_all_authorities::*, error::BoxedError};
use oxidauth_repository::authorities::select_all_authorities::SelectAllAuthoritiesQuery;

pub struct ListAllAuthoritiesUseCase<T>
where
    T: SelectAllAuthoritiesQuery,
{
    authorities: T,
}

impl<T> ListAllAuthoritiesUseCase<T>
where
    T: SelectAllAuthoritiesQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<T> ListAllAuthoritiesServiceTrait for ListAllAuthoritiesUseCase<T>
where
    T: SelectAllAuthoritiesQuery,
{
    #[tracing::instrument(name = "ListAllAuthoritiesUseCase::list_all_authorities", skip(self))]
    async fn list_all_authorities(
        &self,
        req: &ListAllAuthorities,
    ) -> Result<Vec<Authority>, BoxedError> {
        self.authorities
            .select_all_authorities(req)
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
        },
        jwt::EntitlementsEncoding,
    };
    use uuid::Uuid;

    use super::*;

    fn authority(name: &str, strategy: AuthorityStrategy) -> Authority {
        Authority {
            id: Uuid::new_v4(),
            name: name.to_owned(),
            client_key: Uuid::new_v4(),
            status: AuthorityStatus::Enabled,
            strategy,
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
        calls: usize,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn calls(log: &SharedLog) -> usize {
        log.lock().expect("log").calls
    }

    struct MockSelectAll {
        log: SharedLog,
        outcome: Outcome,
    }

    enum Outcome {
        Rows,
        Empty,
        Fail,
    }

    #[async_trait]
    impl SelectAllAuthoritiesQuery for MockSelectAll {
        async fn select_all_authorities(
            &self,
            params: &ListAllAuthorities,
        ) -> Result<Vec<Authority>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .calls += 1;

            match self.outcome {
                Outcome::Rows => {
                    Ok(vec![
                        authority("username-password", AuthorityStrategy::UsernamePassword),
                        authority("oauth2", AuthorityStrategy::Oauth2),
                    ])
                },
                Outcome::Empty => Ok(vec![]),
                Outcome::Fail => Err("simulated database failure".into()),
            }
        }
    }

    fn use_case(log: &SharedLog, outcome: Outcome) -> ListAllAuthoritiesUseCase<MockSelectAll> {
        ListAllAuthoritiesUseCase::new(MockSelectAll {
            log: log.clone(),
            outcome,
        })
    }

    #[tokio::test]
    async fn rows_pass_through_in_repository_order() {
        let log = Arc::new(Mutex::new(Log::default()));
        let req = ListAllAuthorities {};

        let authorities = use_case(&log, Outcome::Rows)
            .list_all_authorities(&req)
            .await
            .expect("list succeeds");

        let names: Vec<&str> = authorities
            .iter()
            .map(|a| a.name.as_str())
            .collect();
        assert_eq!(names, vec!["username-password", "oauth2"]);
        assert_eq!(calls(&log), 1);
    }

    #[tokio::test]
    async fn an_empty_table_lists_as_an_empty_vec() {
        let log = Arc::new(Mutex::new(Log::default()));
        let req = ListAllAuthorities {};

        let authorities = use_case(&log, Outcome::Empty)
            .list_all_authorities(&req)
            .await
            .expect("empty list is not an error");

        assert!(authorities.is_empty());
    }

    #[tokio::test]
    async fn database_failure_propagates() {
        let log = Arc::new(Mutex::new(Log::default()));
        let req = ListAllAuthorities {};

        let error = use_case(&log, Outcome::Fail)
            .list_all_authorities(&req)
            .await
            .expect_err("database failure surfaces");

        assert_eq!(error.to_string(), "simulated database failure");
    }
}
