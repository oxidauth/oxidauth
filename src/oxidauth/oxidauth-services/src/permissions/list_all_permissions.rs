use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, permissions::list_all_permissions::*};
use oxidauth_repository::permissions::select_all_permissions::SelectAllPermissionsQuery;

pub struct ListAllPermissionsUseCase<T>
where
    T: SelectAllPermissionsQuery,
{
    permissions: T,
}

impl<T> ListAllPermissionsUseCase<T>
where
    T: SelectAllPermissionsQuery,
{
    pub fn new(permissions: T) -> Self {
        Self { permissions }
    }
}

#[async_trait]
impl<T> ListAllPermissionsServiceTrait for ListAllPermissionsUseCase<T>
where
    T: SelectAllPermissionsQuery,
{
    #[tracing::instrument(name = "ListAllPermissionsUseCase::list_all_permissions", skip(self))]
    async fn list_all_permissions(
        &self,
        req: &ListAllPermissions,
    ) -> Result<Vec<Permission>, BoxedError> {
        self.permissions
            .select_all_permissions(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;

    use super::*;

    fn stored_permissions() -> Vec<Permission> {
        [("users", "read"), ("users", "write")]
            .iter()
            .map(|(resource, action)| {
                Permission {
                    id: uuid::uuid!("11111111-1111-4111-8111-111111111111"),
                    realm: "oxidauth".to_owned(),
                    resource: (*resource).to_owned(),
                    action: (*action).to_owned(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }
            })
            .collect()
    }

    #[derive(Default)]
    struct Log {
        calls: usize,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn calls(log: &SharedLog) -> usize {
        log.lock().expect("log").calls
    }

    struct MockPermissions {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl SelectAllPermissionsQuery for MockPermissions {
        async fn select_all_permissions(
            &self,
            params: &ListAllPermissions,
        ) -> Result<Vec<Permission>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .calls += 1;

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(stored_permissions())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> ListAllPermissionsUseCase<MockPermissions> {
        ListAllPermissionsUseCase::new(MockPermissions {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate: query order passes through with no re-sort/dedup.
    #[tokio::test]
    async fn the_stored_list_is_returned_in_query_order() {
        let log = Arc::new(Mutex::new(Log::default()));

        let permissions = use_case(&log, false)
            .list_all_permissions(&ListAllPermissions)
            .await
            .expect("list succeeds");

        assert_eq!(calls(&log), 1);
        assert_eq!(
            permissions
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>(),
            vec!["oxidauth:users:read", "oxidauth:users:write"]
        );
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .list_all_permissions(&ListAllPermissions)
            .await
            .expect_err("select failure propagates");

        assert_eq!(error.to_string(), "simulated select failure");
    }
}
