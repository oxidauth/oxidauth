use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, roles::delete_role::*};
use oxidauth_repository::roles::delete_role::DeleteRoleQuery;

pub struct DeleteRoleUseCase<T>
where
    T: DeleteRoleQuery,
{
    roles: T,
}

impl<T> DeleteRoleUseCase<T>
where
    T: DeleteRoleQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<T> DeleteRoleServiceTrait for DeleteRoleUseCase<T>
where
    T: DeleteRoleQuery,
{
    #[tracing::instrument(name = "DeleteRoleUseCase::delete_role", skip(self))]
    async fn delete_role(&self, req: &DeleteRole) -> Result<Role, BoxedError> {
        self.roles
            .delete_role(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use uuid::Uuid;

    use super::*;

    fn role_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    fn deleted_role() -> Role {
        Role {
            id: role_id(),
            name: "admin".to_owned(),
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

    struct MockRoles {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl DeleteRoleQuery for MockRoles {
        async fn delete_role(&self, req: &DeleteRole) -> Result<Role, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .deletes
                .push(req.role_id);

            if self.fail {
                return Err("simulated delete failure".into());
            }

            Ok(deleted_role())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> DeleteRoleUseCase<MockRoles> {
        DeleteRoleUseCase::new(MockRoles {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate; FK-referenced roles (role_role_grants /
    // role_permission_grants) fail inside the query and must surface raw.
    #[tokio::test]
    async fn id_reaches_the_query_and_the_deleted_row_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));

        let role = use_case(&log, false)
            .delete_role(&DeleteRole { role_id: role_id() })
            .await
            .expect("delete succeeds");

        assert_eq!(deletes(&log), vec![role_id()]);
        assert_eq!(role.id, role_id());
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .delete_role(&DeleteRole { role_id: role_id() })
            .await
            .expect_err("FK-violation error propagates unchanged");

        assert_eq!(error.to_string(), "simulated delete failure");
    }
}
