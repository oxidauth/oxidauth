use async_trait::async_trait;
use oxidauth_kernel::{authorities::delete_authority::*, error::BoxedError};
use oxidauth_repository::authorities::delete_authority::DeleteAuthorityQuery;

pub struct DeleteAuthorityUseCase<T>
where
    T: DeleteAuthorityQuery,
{
    authorities: T,
}

impl<T> DeleteAuthorityUseCase<T>
where
    T: DeleteAuthorityQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<T> DeleteAuthorityServiceTrait for DeleteAuthorityUseCase<T>
where
    T: DeleteAuthorityQuery,
{
    #[tracing::instrument(name = "DeleteAuthorityUseCase::delete_authority", skip(self))]
    async fn delete_authority(&self, req: &DeleteAuthority) -> Result<Authority, BoxedError> {
        self.authorities
            .delete_authority(req)
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
        uuid::uuid!("44444444-4444-4444-8444-444444444444")
    }

    fn authority(id: Uuid) -> Authority {
        Authority {
            id,
            name: "deleted-authority".to_owned(),
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
        deletes: Vec<Uuid>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn deletes(log: &SharedLog) -> Vec<Uuid> {
        log.lock()
            .expect("log")
            .deletes
            .clone()
    }

    struct MockDelete {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl DeleteAuthorityQuery for MockDelete {
        async fn delete_authority(&self, req: &DeleteAuthority) -> Result<Authority, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .deletes
                .push(req.authority_id);

            if self.fail {
                return Err("RowNotFound".into());
            }

            Ok(authority(req.authority_id))
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> DeleteAuthorityUseCase<MockDelete> {
        DeleteAuthorityUseCase::new(MockDelete {
            log: log.clone(),
            fail,
        })
    }

    #[tokio::test]
    async fn delete_returns_the_removed_authority() {
        let log = Arc::new(Mutex::new(Log::default()));
        let req = DeleteAuthority {
            authority_id: authority_id(),
        };

        let deleted = use_case(&log, false)
            .delete_authority(&req)
            .await
            .expect("delete succeeds");

        assert_eq!(deleted.id, authority_id());
        assert_eq!(deletes(&log), vec![authority_id()]);
    }

    #[tokio::test]
    async fn missing_authority_error_surfaces_verbatim() {
        let log = Arc::new(Mutex::new(Log::default()));
        let req = DeleteAuthority {
            authority_id: authority_id(),
        };

        // The use case adds no not-found mapping of its own — the repository
        // RowNotFound (or an FK violation on user_authorities) surfaces as-is.
        let error = use_case(&log, true)
            .delete_authority(&req)
            .await
            .expect_err("repository failure surfaces");

        assert_eq!(error.to_string(), "RowNotFound");
        assert_eq!(deletes(&log), vec![authority_id()]);
    }
}
