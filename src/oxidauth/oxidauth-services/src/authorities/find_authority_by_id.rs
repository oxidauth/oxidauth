use async_trait::async_trait;
use oxidauth_kernel::{authorities::find_authority_by_id::*, error::BoxedError};
use oxidauth_repository::authorities::select_authority_by_id::SelectAuthorityByIdQuery;

pub struct FindAuthorityByIdUseCase<T>
where
    T: SelectAuthorityByIdQuery,
{
    authorities: T,
}

impl<T> FindAuthorityByIdUseCase<T>
where
    T: SelectAuthorityByIdQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<T> FindAuthorityByIdServiceTrait for FindAuthorityByIdUseCase<T>
where
    T: SelectAuthorityByIdQuery,
{
    #[tracing::instrument(name = "FindAuthorityByIdUseCase::find_authority_by_id", skip(self))]
    async fn find_authority_by_id(&self, req: &FindAuthorityById) -> Result<Authority, BoxedError> {
        self.authorities
            .select_authority_by_id(req)
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
        authorities::{AuthoritySettings, AuthorityStatus, AuthorityStrategy, TotpSettings},
        jwt::EntitlementsEncoding,
    };
    use uuid::Uuid;

    use super::*;

    fn authority_id() -> Uuid {
        uuid::uuid!("55555555-5555-5555-8555-555555555555")
    }

    fn authority() -> Authority {
        Authority {
            id: authority_id(),
            name: "found-authority".to_owned(),
            client_key: uuid::Uuid::new_v4(),
            status: AuthorityStatus::Enabled,
            strategy: AuthorityStrategy::UsernamePassword,
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
        finds: Vec<Uuid>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn finds(log: &SharedLog) -> Vec<Uuid> {
        log.lock()
            .expect("log")
            .finds
            .clone()
    }

    struct MockSelectById {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl SelectAuthorityByIdQuery for MockSelectById {
        async fn select_authority_by_id(
            &self,
            req: &FindAuthorityById,
        ) -> Result<Authority, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .finds
                .push(req.authority_id);

            if self.fail {
                return Err("RowNotFound".into());
            }

            Ok(authority())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> FindAuthorityByIdUseCase<MockSelectById> {
        FindAuthorityByIdUseCase::new(MockSelectById {
            log: log.clone(),
            fail,
        })
    }

    #[tokio::test]
    async fn found_authority_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));
        let req = FindAuthorityById {
            authority_id: authority_id(),
        };

        let found = use_case(&log, false)
            .find_authority_by_id(&req)
            .await
            .expect("find succeeds");

        assert_eq!(found.id, authority_id());
        assert_eq!(found.name, "found-authority");
        assert_eq!(finds(&log), vec![authority_id()]);
    }

    #[tokio::test]
    async fn missing_authority_error_surfaces_verbatim() {
        let log = Arc::new(Mutex::new(Log::default()));
        let req = FindAuthorityById {
            authority_id: authority_id(),
        };

        // No service-level not-found mapping here (unlike find-by-strategy):
        // the repository's RowNotFound error surfaces as-is.
        let error = use_case(&log, true)
            .find_authority_by_id(&req)
            .await
            .expect_err("missing row surfaces");

        assert_eq!(error.to_string(), "RowNotFound");
        assert_eq!(finds(&log), vec![authority_id()]);
    }
}
