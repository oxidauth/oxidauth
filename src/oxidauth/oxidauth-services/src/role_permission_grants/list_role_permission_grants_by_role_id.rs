use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    role_permission_grants::list_role_permission_grants_by_role_id::*,
};
use oxidauth_repository::role_permission_grants::select_role_permission_grants_by_role_id::SelectRolePermissionGrantsByRoleIdQuery;

pub struct ListRolePermissionGrantsByRoleIdUseCase<T>
where
    T: SelectRolePermissionGrantsByRoleIdQuery,
{
    role_permission_grants: T,
}

impl<T> ListRolePermissionGrantsByRoleIdUseCase<T>
where
    T: SelectRolePermissionGrantsByRoleIdQuery,
{
    pub fn new(role_permission_grants: T) -> Self {
        Self {
            role_permission_grants,
        }
    }
}

#[async_trait]
impl<T> ListRolePermissionGrantsByRoleIdServiceTrait for ListRolePermissionGrantsByRoleIdUseCase<T>
where
    T: SelectRolePermissionGrantsByRoleIdQuery,
{
    #[tracing::instrument(
        name = "ListRolePermissionGrantsByRoleIdUseCase::list_role_permission_grants_by_role_id",
        skip(self)
    )]
    async fn list_role_permission_grants_by_role_id(
        &self,
        req: &ListRolePermissionGrantsByRoleId,
    ) -> Result<Vec<RolePermission>, BoxedError> {
        self.role_permission_grants
            .select_role_permission_grants_by_role_id(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::{
        permissions::Permission,
        role_permission_grants::{RolePermission, RolePermissionGrant},
    };
    use uuid::Uuid;

    use super::*;

    fn role_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    fn role_permission(action: &str) -> RolePermission {
        RolePermission {
            permission: Permission {
                id: uuid::uuid!("33333333-3333-4333-8333-333333333333"),
                realm: "oxidauth".to_owned(),
                resource: "users".to_owned(),
                action: action.to_owned(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            grant: RolePermissionGrant {
                role_id: role_id(),
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
    impl SelectRolePermissionGrantsByRoleIdQuery for MockGrants {
        async fn select_role_permission_grants_by_role_id(
            &self,
            req: &ListRolePermissionGrantsByRoleId,
        ) -> Result<Vec<RolePermission>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .calls
                .push(req.role_id);

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(self
                .actions
                .iter()
                .map(|a| role_permission(a))
                .collect())
        }
    }

    fn use_case(
        log: &SharedLog,
        actions: Vec<String>,
        fail: bool,
    ) -> ListRolePermissionGrantsByRoleIdUseCase<MockGrants> {
        ListRolePermissionGrantsByRoleIdUseCase::new(MockGrants {
            log: log.clone(),
            actions,
            fail,
        })
    }

    // Pure delegate: the permission join is done in SQL; relayed untouched,
    // empty included.
    #[tokio::test]
    async fn role_id_reaches_the_query_and_the_join_is_relayed() {
        let log = Arc::new(Mutex::new(Log::default()));

        let rows = use_case(&log, vec!["read".to_owned(), "write".to_owned()], false)
            .list_role_permission_grants_by_role_id(&ListRolePermissionGrantsByRoleId {
                role_id: role_id(),
            })
            .await
            .expect("list succeeds");

        assert_eq!(calls(&log), vec![role_id()]);
        assert_eq!(
            rows.iter()
                .map(|r| r.permission.action.as_str())
                .collect::<Vec<_>>(),
            vec!["read", "write"]
        );

        let log = Arc::new(Mutex::new(Log::default()));
        let rows = use_case(&log, vec![], false)
            .list_role_permission_grants_by_role_id(&ListRolePermissionGrantsByRoleId {
                role_id: role_id(),
            })
            .await
            .expect("a role without permissions is not an error");

        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, vec![], true)
            .list_role_permission_grants_by_role_id(&ListRolePermissionGrantsByRoleId {
                role_id: role_id(),
            })
            .await
            .expect_err("select failure propagates");

        assert_eq!(error.to_string(), "simulated select failure");
    }
}
