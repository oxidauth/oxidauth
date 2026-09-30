use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, permissions::delete_permission::*};
use oxidauth_repository::permissions::delete_permission::DeletePermissionQuery;

pub struct DeletePermissionUseCase<T>
where
    T: DeletePermissionQuery,
{
    permissions: T,
}

impl<T> DeletePermissionUseCase<T>
where
    T: DeletePermissionQuery,
{
    pub fn new(permissions: T) -> Self {
        Self { permissions }
    }
}

#[async_trait]
impl<T> DeletePermissionServiceTrait for DeletePermissionUseCase<T>
where
    T: DeletePermissionQuery,
{
    #[tracing::instrument(name = "DeletePermissionUseCase::delete_permission", skip(self))]
    async fn delete_permission(&self, req: &DeletePermission) -> Result<Permission, BoxedError> {
        self.permissions
            .delete_permission(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;

    use super::*;

    fn deleted_permission() -> Permission {
        Permission {
            id: uuid::uuid!("11111111-1111-4111-8111-111111111111"),
            realm: "oxidauth".to_owned(),
            resource: "users".to_owned(),
            action: "read".to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Default)]
    struct Log {
        deletes: Vec<String>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn deletes(log: &SharedLog) -> Vec<String> {
        log.lock()
            .expect("log")
            .deletes
            .clone()
    }

    struct MockPermissions {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl DeletePermissionQuery for MockPermissions {
        async fn delete_permission(
            &self,
            req: &DeletePermission,
        ) -> Result<Permission, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .deletes
                .push(req.permission.clone());

            if self.fail {
                return Err("simulated delete failure".into());
            }

            Ok(deleted_permission())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> DeletePermissionUseCase<MockPermissions> {
        DeletePermissionUseCase::new(MockPermissions {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate; FK-referenced permissions (role_permission_grants)
    // fail inside the query and must surface raw.
    #[tokio::test]
    async fn permission_string_reaches_the_query_and_the_row_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));

        let permission = use_case(&log, false)
            .delete_permission(&DeletePermission {
                permission: "oxidauth:users:read".to_owned(),
            })
            .await
            .expect("delete succeeds");

        assert_eq!(deletes(&log), vec!["oxidauth:users:read".to_owned()]);
        assert_eq!(permission.to_string(), "oxidauth:users:read");
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .delete_permission(&DeletePermission {
                permission: "oxidauth:users:read".to_owned(),
            })
            .await
            .expect_err("FK-violation error propagates unchanged");

        assert_eq!(error.to_string(), "simulated delete failure");
    }
}
