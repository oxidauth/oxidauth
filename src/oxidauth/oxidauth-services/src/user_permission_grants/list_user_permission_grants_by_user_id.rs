use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    user_permission_grants::list_user_permission_grants_by_user_id::*,
};
use oxidauth_repository::user_permission_grants::select_user_permission_grants_by_user_id::SelectUserPermissionGrantsByUserIdQuery;

pub struct ListUserPermissionGrantsByUserIdUseCase<T>
where
    T: SelectUserPermissionGrantsByUserIdQuery,
{
    user_permission_grants: T,
}

impl<T> ListUserPermissionGrantsByUserIdUseCase<T>
where
    T: SelectUserPermissionGrantsByUserIdQuery,
{
    pub fn new(user_permission_grants: T) -> Self {
        Self {
            user_permission_grants,
        }
    }
}

#[async_trait]
impl<T> ListUserPermissionGrantsByUserIdServiceTrait for ListUserPermissionGrantsByUserIdUseCase<T>
where
    T: SelectUserPermissionGrantsByUserIdQuery,
{
    #[tracing::instrument(
        name = "ListUserPermissionGrantsByUserIdUseCase::list_user_permission_grants_by_user_id",
        skip(self)
    )]
    async fn list_user_permission_grants_by_user_id(
        &self,
        req: &ListUserPermissionGrantsByUserId,
    ) -> Result<Vec<UserPermission>, BoxedError> {
        self.user_permission_grants
            .select_user_permission_grants_by_user_id(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::{
        permissions::Permission,
        user_permission_grants::{UserPermission, UserPermissionGrant},
    };
    use uuid::Uuid;

    use super::*;

    fn user_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    fn user_permission(action: &str) -> UserPermission {
        UserPermission {
            permission: Permission {
                id: uuid::uuid!("33333333-3333-4333-8333-333333333333"),
                realm: "oxidauth".to_owned(),
                resource: "users".to_owned(),
                action: action.to_owned(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            grant: UserPermissionGrant {
                user_id: user_id(),
                permission_id: uuid::uuid!("33333333-3333-4333-8333-333333333333"),
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
        actions: Vec<String>,
        fail: bool,
    }

    #[async_trait]
    impl SelectUserPermissionGrantsByUserIdQuery for MockGrants {
        async fn select_user_permission_grants_by_user_id(
            &self,
            req: &ListUserPermissionGrantsByUserId,
        ) -> Result<Vec<UserPermission>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .calls
                .push(req.user_id);

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(self
                .actions
                .iter()
                .map(|a| user_permission(a))
                .collect())
        }
    }

    fn use_case(
        log: &SharedLog,
        actions: Vec<String>,
        fail: bool,
    ) -> ListUserPermissionGrantsByUserIdUseCase<MockGrants> {
        ListUserPermissionGrantsByUserIdUseCase::new(MockGrants {
            log: log.clone(),
            actions,
            fail,
        })
    }

    // Pure delegate: direct-grant join relayed untouched, empty included.
    #[tokio::test]
    async fn user_id_reaches_the_query_and_the_join_is_relayed() {
        let log = Arc::new(Mutex::new(Log::default()));

        let rows = use_case(&log, vec!["read".to_owned()], false)
            .list_user_permission_grants_by_user_id(&ListUserPermissionGrantsByUserId {
                user_id: user_id(),
            })
            .await
            .expect("list succeeds");

        assert_eq!(calls(&log), vec![user_id()]);
        assert_eq!(rows[0].permission.action, "read");

        let log = Arc::new(Mutex::new(Log::default()));
        let rows = use_case(&log, vec![], false)
            .list_user_permission_grants_by_user_id(&ListUserPermissionGrantsByUserId {
                user_id: user_id(),
            })
            .await
            .expect("a user without direct grants is not an error");

        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, vec![], true)
            .list_user_permission_grants_by_user_id(&ListUserPermissionGrantsByUserId {
                user_id: user_id(),
            })
            .await
            .expect_err("select failure propagates");

        assert_eq!(error.to_string(), "simulated select failure");
    }
}
