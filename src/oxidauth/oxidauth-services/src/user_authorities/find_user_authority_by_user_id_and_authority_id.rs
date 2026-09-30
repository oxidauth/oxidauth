use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    user_authorities::find_user_authority_by_user_id_and_authority_id::*,
};
use oxidauth_repository::user_authorities::select_user_authority_by_user_id_and_authority_id::SelectUserAuthorityByUserIdAndAuthorityIdQuery;

pub struct FindUserAuthorityByUserIdAndAuthorityIdUseCase<T>
where
    T: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
{
    user_authorities: T,
}

impl<T> FindUserAuthorityByUserIdAndAuthorityIdUseCase<T>
where
    T: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
{
    pub fn new(user_authorities: T) -> Self {
        Self { user_authorities }
    }
}

#[async_trait]
impl<T> FindUserAuthorityByUserIdAndAuthorityIdServiceTrait
    for FindUserAuthorityByUserIdAndAuthorityIdUseCase<T>
where
    T: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
{
    #[tracing::instrument(
        name = "FindUserAuthorityByUserIdAndAuthorityIdUseCase::find_user_authority_by_user_id_and_authority_id",
        skip(self)
    )]
    async fn find_user_authority_by_user_id_and_authority_id(
        &self,
        req: &FindUserAuthorityByUserIdAndAuthorityId,
    ) -> Result<UserAuthorityWithAuthority, BoxedError> {
        self.user_authorities
            .select_user_authority_by_user_id_and_authority_id(req)
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
        user_authorities::UserAuthority,
    };
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn user_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    fn authority_id() -> Uuid {
        uuid::uuid!("22222222-2222-4222-8222-222222222222")
    }

    fn row() -> UserAuthorityWithAuthority {
        UserAuthorityWithAuthority {
            user_authority: UserAuthority {
                user_id: user_id(),
                authority_id: authority_id(),
                user_identifier: "octocat".to_owned(),
                params: JsonValue::new(json!({"password_hash": "…"})),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            authority: Authority {
                id: authority_id(),
                name: "default".to_owned(),
                client_key: uuid::uuid!("33333333-3333-4333-8333-333333333333"),
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
            },
        }
    }

    #[derive(Default)]
    struct Log {
        calls: Vec<(Uuid, Uuid)>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn calls(log: &SharedLog) -> Vec<(Uuid, Uuid)> {
        log.lock()
            .expect("log")
            .calls
            .clone()
    }

    struct MockUserAuthorities {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl SelectUserAuthorityByUserIdAndAuthorityIdQuery for MockUserAuthorities {
        async fn select_user_authority_by_user_id_and_authority_id(
            &self,
            req: &FindUserAuthorityByUserIdAndAuthorityId,
        ) -> Result<UserAuthorityWithAuthority, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .calls
                .push((req.user_id, req.authority_id));

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(row())
        }
    }

    fn use_case(
        log: &SharedLog,
        fail: bool,
    ) -> FindUserAuthorityByUserIdAndAuthorityIdUseCase<MockUserAuthorities> {
        FindUserAuthorityByUserIdAndAuthorityIdUseCase::new(MockUserAuthorities {
            log: log.clone(),
            fail,
        })
    }

    fn request() -> FindUserAuthorityByUserIdAndAuthorityId {
        FindUserAuthorityByUserIdAndAuthorityId {
            user_id: user_id(),
            authority_id: authority_id(),
        }
    }

    // Pure delegate: the joined (user_authority, authority) pair is
    // assembled in SQL and relayed untouched.
    #[tokio::test]
    async fn the_pair_reaches_the_query_and_the_join_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));

        let result = use_case(&log, false)
            .find_user_authority_by_user_id_and_authority_id(&request())
            .await
            .expect("find succeeds");

        assert_eq!(calls(&log), vec![(user_id(), authority_id())]);
        assert_eq!(
            result
                .user_authority
                .user_identifier,
            "octocat"
        );
        assert_eq!(result.authority.id, authority_id());
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .find_user_authority_by_user_id_and_authority_id(&request())
            .await
            .expect_err("missing-row error propagates unchanged");

        assert_eq!(error.to_string(), "simulated select failure");
    }
}
