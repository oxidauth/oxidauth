use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, user_authorities::list_user_authorities_by_user_id::*};
use oxidauth_repository::user_authorities::select_user_authorities_by_user_id::SelectUserAuthoritiesByUserIdQuery;

pub struct ListUserAuthoritiesByUserIdUseCase<T>
where
    T: SelectUserAuthoritiesByUserIdQuery,
{
    user_authorities: T,
}

impl<T> ListUserAuthoritiesByUserIdUseCase<T>
where
    T: SelectUserAuthoritiesByUserIdQuery,
{
    pub fn new(user_authorities: T) -> Self {
        Self { user_authorities }
    }
}

#[async_trait]
impl<T> ListUserAuthoritiesByUserIdServiceTrait for ListUserAuthoritiesByUserIdUseCase<T>
where
    T: SelectUserAuthoritiesByUserIdQuery,
{
    #[tracing::instrument(
        name = "ListUserAuthoritiesByUserIdUseCase::list_user_authorities_by_user_id",
        skip(self)
    )]
    async fn list_user_authorities_by_user_id(
        &self,
        req: &ListUserAuthoritiesByUserId,
    ) -> Result<Vec<UserAuthorityWithAuthority>, BoxedError> {
        self.user_authorities
            .select_user_authorities_by_user_id(req)
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

    fn row(identifier: &str) -> UserAuthorityWithAuthority {
        UserAuthorityWithAuthority {
            user_authority: UserAuthority {
                user_id: user_id(),
                authority_id: uuid::uuid!("22222222-2222-4222-8222-222222222222"),
                user_identifier: identifier.to_owned(),
                params: JsonValue::new(json!({})),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            authority: Authority {
                id: uuid::uuid!("22222222-2222-4222-8222-222222222222"),
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
        calls: Vec<Uuid>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn calls(log: &SharedLog) -> Vec<Uuid> {
        log.lock()
            .expect("log")
            .calls
            .clone()
    }

    struct MockUserAuthorities {
        log: SharedLog,
        identifiers: Vec<String>,
        fail: bool,
    }

    #[async_trait]
    impl SelectUserAuthoritiesByUserIdQuery for MockUserAuthorities {
        async fn select_user_authorities_by_user_id(
            &self,
            req: &ListUserAuthoritiesByUserId,
        ) -> Result<Vec<UserAuthorityWithAuthority>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .calls
                .push(req.user_id);

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(self
                .identifiers
                .iter()
                .map(|i| row(i))
                .collect())
        }
    }

    fn use_case(
        log: &SharedLog,
        identifiers: Vec<String>,
        fail: bool,
    ) -> ListUserAuthoritiesByUserIdUseCase<MockUserAuthorities> {
        ListUserAuthoritiesByUserIdUseCase::new(MockUserAuthorities {
            log: log.clone(),
            identifiers,
            fail,
        })
    }

    // Pure delegate: joined rows relayed untouched, empty included.
    #[tokio::test]
    async fn user_id_reaches_the_query_and_the_join_is_relayed() {
        let log = Arc::new(Mutex::new(Log::default()));

        let rows = use_case(&log, vec!["octocat".to_owned()], false)
            .list_user_authorities_by_user_id(&ListUserAuthoritiesByUserId { user_id: user_id() })
            .await
            .expect("list succeeds");

        assert_eq!(calls(&log), vec![user_id()]);
        assert_eq!(
            rows[0]
                .user_authority
                .user_identifier,
            "octocat"
        );

        let log = Arc::new(Mutex::new(Log::default()));
        let rows = use_case(&log, vec![], false)
            .list_user_authorities_by_user_id(&ListUserAuthoritiesByUserId { user_id: user_id() })
            .await
            .expect("a user without authorities is not an error");

        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, vec![], true)
            .list_user_authorities_by_user_id(&ListUserAuthoritiesByUserId { user_id: user_id() })
            .await
            .expect_err("select failure propagates");

        assert_eq!(error.to_string(), "simulated select failure");
    }
}
