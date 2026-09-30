use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, user_authorities::delete_user_authority::*};
use oxidauth_repository::user_authorities::delete_user_authority::DeleteUserAuthorityQuery;

pub struct DeleteUserAuthorityUseCase<T>
where
    T: DeleteUserAuthorityQuery,
{
    user_authorities: T,
}

impl<T> DeleteUserAuthorityUseCase<T>
where
    T: DeleteUserAuthorityQuery,
{
    pub fn new(user_authorities: T) -> Self {
        Self { user_authorities }
    }
}

#[async_trait]
impl<T> DeleteUserAuthorityServiceTrait for DeleteUserAuthorityUseCase<T>
where
    T: DeleteUserAuthorityQuery,
{
    #[tracing::instrument(name = "DeleteUserAuthorityUseCase::delete_user_authority", skip(self))]
    async fn delete_user_authority(
        &self,
        req: &DeleteUserAuthority,
    ) -> Result<UserAuthority, BoxedError> {
        self.user_authorities
            .delete_user_authority(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::JsonValue;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn user_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    fn authority_id() -> Uuid {
        uuid::uuid!("22222222-2222-4222-8222-222222222222")
    }

    #[derive(Default)]
    struct Log {
        deletes: Vec<(Uuid, Uuid)>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn deletes(log: &SharedLog) -> Vec<(Uuid, Uuid)> {
        log.lock()
            .expect("log")
            .deletes
            .clone()
    }

    struct MockUserAuthorities {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl DeleteUserAuthorityQuery for MockUserAuthorities {
        async fn delete_user_authority(
            &self,
            req: &DeleteUserAuthority,
        ) -> Result<UserAuthority, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .deletes
                .push((req.user_id, req.authority_id));

            if self.fail {
                return Err("simulated delete failure".into());
            }

            Ok(UserAuthority {
                user_id: req.user_id,
                authority_id: req.authority_id,
                user_identifier: "octocat".to_owned(),
                // the params column holds the argon2 password hash —
                // Debug must never expose it (kernel redaction test)
                params: JsonValue::new(json!({"password_hash": "REDACTED-IN-DEBUG"})),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> DeleteUserAuthorityUseCase<MockUserAuthorities> {
        DeleteUserAuthorityUseCase::new(MockUserAuthorities {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate.
    #[tokio::test]
    async fn the_pair_reaches_the_query_and_the_row_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));

        let row = use_case(&log, false)
            .delete_user_authority(&DeleteUserAuthority {
                user_id: user_id(),
                authority_id: authority_id(),
            })
            .await
            .expect("delete succeeds");

        assert_eq!(deletes(&log), vec![(user_id(), authority_id())]);
        assert_eq!(row.user_identifier, "octocat");
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .delete_user_authority(&DeleteUserAuthority {
                user_id: user_id(),
                authority_id: authority_id(),
            })
            .await
            .expect_err("delete failure propagates");

        assert_eq!(error.to_string(), "simulated delete failure");
    }
}
