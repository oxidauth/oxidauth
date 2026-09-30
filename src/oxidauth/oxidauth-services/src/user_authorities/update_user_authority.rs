use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, user_authorities::update_user_authority::*};
use oxidauth_repository::user_authorities::update_user_authority::UpdateUserAuthorityQuery;

pub struct UpdateUserAuthorityUseCase<T>
where
    T: UpdateUserAuthorityQuery,
{
    user_authorities: T,
}

impl<T> UpdateUserAuthorityUseCase<T>
where
    T: UpdateUserAuthorityQuery,
{
    pub fn new(user_authorities: T) -> Self {
        Self { user_authorities }
    }
}

#[async_trait]
impl<T> UpdateUserAuthorityServiceTrait for UpdateUserAuthorityUseCase<T>
where
    T: UpdateUserAuthorityQuery,
{
    #[tracing::instrument(name = "UpdateUserAuthorityUseCase::update_user_authority", skip(self))]
    async fn update_user_authority(
        &self,
        req: &UpdateUserAuthority,
    ) -> Result<UserAuthority, BoxedError> {
        self.user_authorities
            .update_user_authority(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::JsonValue;
    use serde_json::{Value, json};
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
        updates: Vec<(Uuid, Uuid, Value)>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn updates(log: &SharedLog) -> Vec<(Uuid, Uuid, Value)> {
        log.lock()
            .expect("log")
            .updates
            .clone()
    }

    struct MockUserAuthorities {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl UpdateUserAuthorityQuery for MockUserAuthorities {
        async fn update_user_authority(
            &self,
            req: &UpdateUserAuthority,
        ) -> Result<UserAuthority, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .updates
                .push((
                    req.user_id,
                    req.authority_id,
                    req.params
                        .clone()
                        .inner_value(),
                ));

            if self.fail {
                return Err("simulated update failure".into());
            }

            Ok(UserAuthority {
                user_id: req.user_id,
                authority_id: req.authority_id,
                user_identifier: "octocat".to_owned(),
                params: req.params.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> UpdateUserAuthorityUseCase<MockUserAuthorities> {
        UpdateUserAuthorityUseCase::new(MockUserAuthorities {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate: unlike `users::update_user` there is no current-row
    // merge — the params payload replaces the stored one wholesale (A7).
    #[tokio::test]
    async fn the_params_payload_reaches_the_query_untouched() {
        let log = Arc::new(Mutex::new(Log::default()));
        let params = JsonValue::new(json!({"password_hash": "new-hash"}));

        let row = use_case(&log, false)
            .update_user_authority(&UpdateUserAuthority {
                user_id: user_id(),
                authority_id: authority_id(),
                params,
            })
            .await
            .expect("update succeeds");

        assert_eq!(
            updates(&log),
            vec![(
                user_id(),
                authority_id(),
                json!({"password_hash": "new-hash"})
            )]
        );
        assert_eq!(
            row.params.inner_value(),
            json!({"password_hash": "new-hash"})
        );
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .update_user_authority(&UpdateUserAuthority {
                user_id: user_id(),
                authority_id: authority_id(),
                params: JsonValue::new(json!({})),
            })
            .await
            .expect_err("update failure propagates");

        assert_eq!(error.to_string(), "simulated update failure");
    }
}
