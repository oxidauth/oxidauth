use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, permissions::create_permission::*};
use oxidauth_repository::permissions::insert_permission::InsertPermissionQuery;

pub struct CreatePermissionUseCase<T>
where
    T: InsertPermissionQuery,
{
    permissions: T,
}

impl<T> CreatePermissionUseCase<T>
where
    T: InsertPermissionQuery,
{
    pub fn new(permissions: T) -> Self {
        Self { permissions }
    }
}

#[async_trait]
impl<T> CreatePermissionServiceTrait for CreatePermissionUseCase<T>
where
    T: InsertPermissionQuery,
{
    #[tracing::instrument(name = "CreatePermissionUseCase::create_permission", skip(self))]
    async fn create_permission(&self, req: &CreatePermission) -> Result<Permission, BoxedError> {
        self.permissions
            .insert_permission(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;

    use super::*;

    fn stored_permission() -> Permission {
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
        inserts: Vec<String>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn inserts(log: &SharedLog) -> Vec<String> {
        log.lock()
            .expect("log")
            .inserts
            .clone()
    }

    struct MockPermissions {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl InsertPermissionQuery for MockPermissions {
        async fn insert_permission(
            &self,
            req: &CreatePermission,
        ) -> Result<Permission, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .inserts
                .push(req.permission.clone());

            if self.fail {
                return Err("simulated insert failure".into());
            }

            Ok(stored_permission())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> CreatePermissionUseCase<MockPermissions> {
        CreatePermissionUseCase::new(MockPermissions {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate: the raw permission string is not parsed or validated
    // at this layer — the repository owns that.
    #[tokio::test]
    async fn permission_string_reaches_the_insert_query_verbatim() {
        let log = Arc::new(Mutex::new(Log::default()));

        let permission = use_case(&log, false)
            .create_permission(&CreatePermission {
                permission: "oxidauth:users:read".to_owned(),
            })
            .await
            .expect("create succeeds");

        assert_eq!(inserts(&log), vec!["oxidauth:users:read".to_owned()]);
        assert_eq!(permission.to_string(), "oxidauth:users:read");
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .create_permission(&CreatePermission {
                permission: "oxidauth:users:read".to_owned(),
            })
            .await
            .expect_err("duplicate-triple insert failure propagates");

        assert_eq!(error.to_string(), "simulated insert failure");
    }
}
