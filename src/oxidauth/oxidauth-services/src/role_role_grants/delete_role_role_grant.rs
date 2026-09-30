use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, role_role_grants::delete_role_role_grant::*};
use oxidauth_repository::role_role_grants::delete_role_role_grant::DeleteRoleRoleGrantQuery;

pub struct DeleteRoleRoleGrantUseCase<T>
where
    T: DeleteRoleRoleGrantQuery,
{
    role_role_grants: T,
}

impl<T> DeleteRoleRoleGrantUseCase<T>
where
    T: DeleteRoleRoleGrantQuery,
{
    pub fn new(role_role_grants: T) -> Self {
        Self { role_role_grants }
    }
}

#[async_trait]
impl<T> DeleteRoleRoleGrantServiceTrait for DeleteRoleRoleGrantUseCase<T>
where
    T: DeleteRoleRoleGrantQuery,
{
    #[tracing::instrument(
        name = "DeleteRoleRoleGrantUseCase::delete_role_role_grant",
        skip(self)
    )]
    async fn delete_role_role_grant(
        &self,
        req: &DeleteRoleRoleGrant,
    ) -> Result<RoleRoleGrant, BoxedError> {
        self.role_role_grants
            .delete_role_role_grant(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::role_role_grants::RoleRoleGrant;
    use uuid::Uuid;

    use super::*;

    fn parent_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    fn child_id() -> Uuid {
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

    struct MockGrants {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl DeleteRoleRoleGrantQuery for MockGrants {
        async fn delete_role_role_grant(
            &self,
            req: &DeleteRoleRoleGrant,
        ) -> Result<RoleRoleGrant, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .deletes
                .push((req.parent_id, req.child_id));

            if self.fail {
                return Err("simulated grant delete failure".into());
            }

            Ok(RoleRoleGrant {
                parent_id: req.parent_id,
                child_id: req.child_id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> DeleteRoleRoleGrantUseCase<MockGrants> {
        DeleteRoleRoleGrantUseCase::new(MockGrants {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate: unlike create, no role existence checks — the pair
    // goes straight to the query (missing pair behavior lives in SQL).
    #[tokio::test]
    async fn the_pair_reaches_the_query_verbatim() {
        let log = Arc::new(Mutex::new(Log::default()));

        let grant = use_case(&log, false)
            .delete_role_role_grant(&DeleteRoleRoleGrant {
                parent_id: parent_id(),
                child_id: child_id(),
            })
            .await
            .expect("delete succeeds");

        assert_eq!(deletes(&log), vec![(parent_id(), child_id())]);
        assert_eq!(grant.parent_id, parent_id());
        assert_eq!(grant.child_id, child_id());
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .delete_role_role_grant(&DeleteRoleRoleGrant {
                parent_id: parent_id(),
                child_id: child_id(),
            })
            .await
            .expect_err("delete failure propagates");

        assert_eq!(error.to_string(), "simulated grant delete failure");
    }
}
