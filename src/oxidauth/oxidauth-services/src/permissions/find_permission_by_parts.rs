use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    permissions::{PermissionNotFoundError, find_permission_by_parts::*},
};
use oxidauth_repository::permissions::select_permission_by_parts::SelectPermissionByPartsQuery;

pub struct FindPermissionByPartsUseCase<T>
where
    T: SelectPermissionByPartsQuery,
{
    permissions: T,
}

impl<T> FindPermissionByPartsUseCase<T>
where
    T: SelectPermissionByPartsQuery,
{
    pub fn new(permissions: T) -> Self {
        Self { permissions }
    }
}

#[async_trait]
impl<T> FindPermissionByPartsServiceTrait for FindPermissionByPartsUseCase<T>
where
    T: SelectPermissionByPartsQuery,
{
    #[tracing::instrument(
        name = "FindPermissionByPartsUseCase::find_permission_by_parts",
        skip(self)
    )]
    async fn find_permission_by_parts(
        &self,
        params: &FindPermissionByParts,
    ) -> Result<Permission, BoxedError> {
        let permission = self
            .permissions
            .select_permission_by_parts(params)
            .await?;

        match permission {
            Some(permission) => Ok(permission),
            None => Err(PermissionNotFoundError::new(&params.permission)),
        }
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
        lookups: Vec<String>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn lookups(log: &SharedLog) -> Vec<String> {
        log.lock()
            .expect("log")
            .lookups
            .clone()
    }

    struct MockPermissions {
        log: SharedLog,
        missing: bool,
        fail: bool,
    }

    #[async_trait]
    impl SelectPermissionByPartsQuery for MockPermissions {
        async fn select_permission_by_parts(
            &self,
            req: &FindPermissionByParts,
        ) -> Result<Option<Permission>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .lookups
                .push(req.permission.clone());

            if self.fail {
                return Err("simulated select failure".into());
            }

            if self.missing {
                return Ok(None);
            }

            Ok(Some(stored_permission()))
        }
    }

    fn use_case(
        log: &SharedLog,
        missing: bool,
        fail: bool,
    ) -> FindPermissionByPartsUseCase<MockPermissions> {
        FindPermissionByPartsUseCase::new(MockPermissions {
            log: log.clone(),
            missing,
            fail,
        })
    }

    #[tokio::test]
    async fn parts_reach_the_query_and_the_row_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));

        let permission = use_case(&log, false, false)
            .find_permission_by_parts(&FindPermissionByParts {
                permission: "oxidauth:users:read".to_owned(),
            })
            .await
            .expect("find succeeds");

        assert_eq!(lookups(&log), vec!["oxidauth:users:read".to_owned()]);
        assert_eq!(permission.to_string(), "oxidauth:users:read");
    }

    #[tokio::test]
    async fn missing_row_becomes_a_permission_not_found_error() {
        // the repo layer answers `Ok(None)`; the not-found error is minted
        // here (mirrors the settings fetch contract)
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true, false)
            .find_permission_by_parts(&FindPermissionByParts {
                permission: "oxidauth:users:delete".to_owned(),
            })
            .await
            .expect_err("Ok(None) is mapped to a not-found error");

        assert_eq!(
            error.to_string(),
            "permission with name not found: oxidauth:users:delete"
        );
        // bootstrap downcasts PermissionNotFoundError — pin the type, not just the copy
        assert!(
            error
                .downcast_ref::<PermissionNotFoundError>()
                .is_some()
        );
        assert_eq!(lookups(&log), vec!["oxidauth:users:delete".to_owned()]);
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped_not_as_not_found() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, false, true)
            .find_permission_by_parts(&FindPermissionByParts {
                permission: "oxidauth:users:read".to_owned(),
            })
            .await
            .expect_err("select failure propagates verbatim");

        assert_eq!(error.to_string(), "simulated select failure");
        assert!(
            !error
                .to_string()
                .contains("not found")
        );
    }
}
