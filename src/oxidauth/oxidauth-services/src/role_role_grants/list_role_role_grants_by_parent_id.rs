use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, role_role_grants::list_role_role_grants_by_parent_id::*};
use oxidauth_repository::role_role_grants::select_role_role_grants_by_parent_id::SelectRoleRoleGrantsByParentIdQuery;

pub struct ListRoleRoleGrantsByParentIdUseCase<T>
where
    T: SelectRoleRoleGrantsByParentIdQuery,
{
    role_role_grants: T,
}

impl<T> ListRoleRoleGrantsByParentIdUseCase<T>
where
    T: SelectRoleRoleGrantsByParentIdQuery,
{
    pub fn new(role_role_grants: T) -> Self {
        Self { role_role_grants }
    }
}

#[async_trait]
impl<T> ListRoleRoleGrantsByParentIdServiceTrait for ListRoleRoleGrantsByParentIdUseCase<T>
where
    T: SelectRoleRoleGrantsByParentIdQuery,
{
    #[tracing::instrument(
        name = "ListRoleRoleGrantsByParentIdUseCase::list_role_role_grants_by_parent_id",
        skip(self)
    )]
    async fn list_role_role_grants_by_parent_id(
        &self,
        req: &ListRoleRoleGrantsByParentId,
    ) -> Result<Vec<RoleRoleGrantDetail>, BoxedError> {
        self.role_role_grants
            .select_role_role_grants_by_parent_id(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::{
        role_role_grants::{RoleRoleGrant, RoleRoleGrantDetail},
        roles::Role,
    };
    use uuid::Uuid;

    use super::*;

    fn parent_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    fn detail(child_name: &str) -> RoleRoleGrantDetail {
        RoleRoleGrantDetail {
            role: Role {
                id: uuid::uuid!("22222222-2222-4222-8222-222222222222"),
                name: child_name.to_owned(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            grant: RoleRoleGrant {
                parent_id: parent_id(),
                child_id: uuid::uuid!("22222222-2222-4222-8222-222222222222"),
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
        child_names: Vec<String>,
        fail: bool,
    }

    #[async_trait]
    impl SelectRoleRoleGrantsByParentIdQuery for MockGrants {
        async fn select_role_role_grants_by_parent_id(
            &self,
            req: &ListRoleRoleGrantsByParentId,
        ) -> Result<Vec<RoleRoleGrantDetail>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .calls
                .push(req.parent_id);

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(self
                .child_names
                .iter()
                .map(|n| detail(n))
                .collect())
        }
    }

    fn use_case(
        log: &SharedLog,
        child_names: Vec<String>,
        fail: bool,
    ) -> ListRoleRoleGrantsByParentIdUseCase<MockGrants> {
        ListRoleRoleGrantsByParentIdUseCase::new(MockGrants {
            log: log.clone(),
            child_names,
            fail,
        })
    }

    // Pure delegate: one level of children (the transitive closure is the
    // permission-tree SQL's job, A3), relayed untouched — empty included.
    #[tokio::test]
    async fn parent_id_reaches_the_query_and_the_children_are_relayed() {
        let log = Arc::new(Mutex::new(Log::default()));

        let rows = use_case(&log, vec!["child-a".to_owned()], false)
            .list_role_role_grants_by_parent_id(&ListRoleRoleGrantsByParentId {
                parent_id: parent_id(),
            })
            .await
            .expect("list succeeds");

        assert_eq!(calls(&log), vec![parent_id()]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].role.name, "child-a");

        let log = Arc::new(Mutex::new(Log::default()));
        let rows = use_case(&log, vec![], false)
            .list_role_role_grants_by_parent_id(&ListRoleRoleGrantsByParentId {
                parent_id: parent_id(),
            })
            .await
            .expect("no children is not an error");

        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, vec![], true)
            .list_role_role_grants_by_parent_id(&ListRoleRoleGrantsByParentId {
                parent_id: parent_id(),
            })
            .await
            .expect_err("select failure propagates");

        assert_eq!(error.to_string(), "simulated select failure");
    }
}
