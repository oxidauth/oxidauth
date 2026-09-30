use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, user_role_grants::list_user_role_grants_by_user_id::*};
use oxidauth_repository::user_role_grants::select_user_role_grants_by_user_id::SelectUserRoleGrantsByUserIdQuery;

pub struct ListUserRoleGrantsByUserIdUseCase<T>
where
    T: SelectUserRoleGrantsByUserIdQuery,
{
    user_role_grants: T,
}

impl<T> ListUserRoleGrantsByUserIdUseCase<T>
where
    T: SelectUserRoleGrantsByUserIdQuery,
{
    pub fn new(user_role_grants: T) -> Self {
        Self { user_role_grants }
    }
}

#[async_trait]
impl<T> ListUserRoleGrantsByUserIdServiceTrait for ListUserRoleGrantsByUserIdUseCase<T>
where
    T: SelectUserRoleGrantsByUserIdQuery,
{
    #[tracing::instrument(
        name = "ListUserRoleGrantsByUserIdUseCase::list_user_role_grants_by_user_id",
        skip(self)
    )]
    async fn list_user_role_grants_by_user_id(
        &self,
        req: &ListUserRoleGrantsByUserId,
    ) -> Result<Vec<UserRole>, BoxedError> {
        self.user_role_grants
            .select_user_role_grants_by_user_id(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::{roles::Role, user_role_grants::UserRole};
    use uuid::Uuid;

    use super::*;

    fn user_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    fn user_role(role_name: &str) -> UserRole {
        UserRole {
            role: Role {
                id: uuid::uuid!("22222222-2222-4222-8222-222222222222"),
                name: role_name.to_owned(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            grant: oxidauth_kernel::user_role_grants::UserRoleGrant {
                user_id: user_id(),
                role_id: uuid::uuid!("22222222-2222-4222-8222-222222222222"),
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

    struct MockGrants {
        log: SharedLog,
        role_names: Vec<String>,
        fail: bool,
    }

    #[async_trait]
    impl SelectUserRoleGrantsByUserIdQuery for MockGrants {
        async fn select_user_role_grants_by_user_id(
            &self,
            req: &ListUserRoleGrantsByUserId,
        ) -> Result<Vec<UserRole>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .calls
                .push(req.user_id);

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(self
                .role_names
                .iter()
                .map(|name| user_role(name))
                .collect())
        }
    }

    fn use_case(
        log: &SharedLog,
        role_names: Vec<String>,
        fail: bool,
    ) -> ListUserRoleGrantsByUserIdUseCase<MockGrants> {
        ListUserRoleGrantsByUserIdUseCase::new(MockGrants {
            log: log.clone(),
            role_names,
            fail,
        })
    }

    // Pure delegate: the join (grant rows enriched with role rows) is done
    // in SQL; the service relays the vector untouched, empty included.
    #[tokio::test]
    async fn user_id_reaches_the_query_and_the_join_is_returned_including_empty() {
        let log = Arc::new(Mutex::new(Log::default()));

        let rows = use_case(&log, vec!["admin".to_owned()], false)
            .list_user_role_grants_by_user_id(&ListUserRoleGrantsByUserId { user_id: user_id() })
            .await
            .expect("list succeeds");

        assert_eq!(calls(&log), vec![user_id()]);
        assert_eq!(rows[0].role.name, "admin");

        let log = Arc::new(Mutex::new(Log::default()));
        let rows = use_case(&log, vec![], false)
            .list_user_role_grants_by_user_id(&ListUserRoleGrantsByUserId { user_id: user_id() })
            .await
            .expect("empty grant list is not an error");

        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, vec![], true)
            .list_user_role_grants_by_user_id(&ListUserRoleGrantsByUserId { user_id: user_id() })
            .await
            .expect_err("select failure propagates");

        assert_eq!(error.to_string(), "simulated select failure");
    }
}
