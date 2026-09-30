use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    permissions::{PermissionNotFoundError, find_permission_by_parts::FindPermissionByParts},
    user_permission_grants::{UserPermission, delete_user_permission_grant::*},
    users::find_user_by_id::FindUserById,
};
use oxidauth_repository::{
    permissions::select_permission_by_parts::SelectPermissionByPartsQuery,
    user_permission_grants::delete_user_permission_grant::DeleteUserPermissionGrantQuery,
    users::select_user_by_id_query::SelectUserByIdQuery,
};

pub struct DeleteUserPermissionGrantUseCase<U, P, UP>
where
    U: SelectUserByIdQuery,
    P: SelectPermissionByPartsQuery,
    UP: DeleteUserPermissionGrantQuery,
{
    users: U,
    permissions: P,
    user_permission_grants: UP,
}

impl<U, P, UP> DeleteUserPermissionGrantUseCase<U, P, UP>
where
    U: SelectUserByIdQuery,
    P: SelectPermissionByPartsQuery,
    UP: DeleteUserPermissionGrantQuery,
{
    pub fn new(users: U, permissions: P, user_permission_grants: UP) -> Self {
        Self {
            users,
            permissions,
            user_permission_grants,
        }
    }
}

#[async_trait]
impl<U, P, UP> DeleteUserPermissionGrantServiceTrait for DeleteUserPermissionGrantUseCase<U, P, UP>
where
    U: SelectUserByIdQuery,
    P: SelectPermissionByPartsQuery,
    UP: DeleteUserPermissionGrantQuery,
{
    #[tracing::instrument(
        name = "DeleteUserPermissionGrantUseCase::delete_user_permission_grant",
        skip(self)
    )]
    async fn delete_user_permission_grant(
        &self,
        req: &DeleteUserPermission,
    ) -> Result<UserPermission, BoxedError> {
        let user = self
            .users
            .select_user_by_id(&FindUserById {
                user_id: req.user_id,
            })
            .await?;

        let permission = self
            .permissions
            .select_permission_by_parts(&FindPermissionByParts {
                permission: req.permission.clone(),
            })
            .await?
            .ok_or_else(|| PermissionNotFoundError::new(&req.permission))?;

        let grant = self
            .user_permission_grants
            .delete_user_permission_grant(&DeleteUserPermissionGrant {
                user_id: user.id,
                permission_id: permission.id,
            })
            .await?;

        Ok(UserPermission { permission, grant })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::{
        permissions::Permission,
        user_permission_grants::UserPermissionGrant,
        users::{User, UserKind, UserStatus},
    };
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn user_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    fn permission_id() -> Uuid {
        uuid::uuid!("33333333-3333-4333-8333-333333333333")
    }

    #[derive(Default)]
    struct Log {
        user_lookups: Vec<Uuid>,
        permission_lookups: Vec<String>,
        deletes: Vec<(Uuid, Uuid)>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn snapshot(log: &SharedLog) -> (Vec<Uuid>, Vec<String>, Vec<(Uuid, Uuid)>) {
        let log = log.lock().expect("log");
        (
            log.user_lookups.clone(),
            log.permission_lookups.clone(),
            log.deletes.clone(),
        )
    }

    struct MockUsers {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl SelectUserByIdQuery for MockUsers {
        async fn select_user_by_id(&self, req: &FindUserById) -> Result<User, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .user_lookups
                .push(req.user_id);

            if self.fail {
                return Err("simulated user lookup failure".into());
            }

            Ok(User {
                id: user_id(),
                kind: UserKind::Human,
                status: UserStatus::Enabled,
                username: "octocat".to_owned(),
                email: None,
                first_name: None,
                last_name: None,
                profile: json!(null),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    struct MockPermissions {
        log: SharedLog,
        missing: bool,
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
                .permission_lookups
                .push(req.permission.clone());

            if self.missing {
                return Ok(None);
            }

            Ok(Some(Permission {
                id: permission_id(),
                realm: "oxidauth".to_owned(),
                resource: "users".to_owned(),
                action: "read".to_owned(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        }
    }

    struct MockGrants {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl DeleteUserPermissionGrantQuery for MockGrants {
        async fn delete_user_permission_grant(
            &self,
            req: &DeleteUserPermissionGrant,
        ) -> Result<UserPermissionGrant, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .deletes
                .push((req.user_id, req.permission_id));

            if self.fail {
                return Err("simulated grant delete failure".into());
            }

            Ok(UserPermissionGrant {
                user_id: req.user_id,
                permission_id: req.permission_id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    fn use_case(
        log: &SharedLog,
        user_fails: bool,
        permission_missing: bool,
        grant_fails: bool,
    ) -> DeleteUserPermissionGrantUseCase<MockUsers, MockPermissions, MockGrants> {
        DeleteUserPermissionGrantUseCase::new(
            MockUsers {
                log: log.clone(),
                fail: user_fails,
            },
            MockPermissions {
                log: log.clone(),
                missing: permission_missing,
            },
            MockGrants {
                log: log.clone(),
                fail: grant_fails,
            },
        )
    }

    fn base_request() -> DeleteUserPermission {
        DeleteUserPermission {
            user_id: user_id(),
            permission: "oxidauth:users:read".to_owned(),
        }
    }

    #[tokio::test]
    async fn the_permission_string_is_resolved_to_an_id_before_the_delete() {
        let log = Arc::new(Mutex::new(Log::default()));

        let result = use_case(&log, false, false, false)
            .delete_user_permission_grant(&base_request())
            .await
            .expect("delete succeeds");

        let (users, permissions, deletes) = snapshot(&log);
        assert_eq!(users, vec![user_id()]);
        assert_eq!(permissions, vec!["oxidauth:users:read".to_owned()]);
        assert_eq!(deletes, vec![(user_id(), permission_id())]);
        assert_eq!(result.permission.to_string(), "oxidauth:users:read");
        assert_eq!(result.grant.permission_id, permission_id());
    }

    #[tokio::test]
    async fn an_unresolvable_permission_aborts_before_the_delete() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, false, true, false)
            .delete_user_permission_grant(&base_request())
            .await
            .expect_err("Ok(None) from the parts lookup becomes not-found");

        assert_eq!(
            error.to_string(),
            "permission with name not found: oxidauth:users:read"
        );
        assert!(snapshot(&log).2.is_empty());
    }

    #[tokio::test]
    async fn a_missing_user_aborts_before_the_permission_lookup() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true, false, false)
            .delete_user_permission_grant(&base_request())
            .await
            .expect_err("user lookup failure propagates");

        assert_eq!(error.to_string(), "simulated user lookup failure");
        let (_, permissions, deletes) = snapshot(&log);
        assert!(permissions.is_empty());
        assert!(deletes.is_empty());
    }

    #[tokio::test]
    async fn grant_delete_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, false, false, true)
            .delete_user_permission_grant(&base_request())
            .await
            .expect_err("delete failure propagates");

        assert_eq!(error.to_string(), "simulated grant delete failure");
    }
}
