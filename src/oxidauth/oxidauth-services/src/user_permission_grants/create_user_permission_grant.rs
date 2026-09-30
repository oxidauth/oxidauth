use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    permissions::{PermissionNotFoundError, find_permission_by_parts::FindPermissionByParts},
    user_permission_grants::{UserPermission, create_user_permission_grant::*},
    users::find_user_by_id::FindUserById,
};
use oxidauth_repository::{
    permissions::select_permission_by_parts::SelectPermissionByPartsQuery,
    user_permission_grants::insert_user_permission_grant::InsertUserPermissionGrantQuery,
    users::select_user_by_id_query::SelectUserByIdQuery,
};

pub struct CreateUserPermissionGrantUseCase<U, P, UP>
where
    U: SelectUserByIdQuery,
    P: SelectPermissionByPartsQuery,
    UP: InsertUserPermissionGrantQuery,
{
    users: U,
    permissions: P,
    user_permission_grants: UP,
}

impl<U, P, UP> CreateUserPermissionGrantUseCase<U, P, UP>
where
    U: SelectUserByIdQuery,
    P: SelectPermissionByPartsQuery,
    UP: InsertUserPermissionGrantQuery,
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
impl<U, P, UP> CreateUserPermissionGrantServiceTrait for CreateUserPermissionGrantUseCase<U, P, UP>
where
    U: SelectUserByIdQuery,
    P: SelectPermissionByPartsQuery,
    UP: InsertUserPermissionGrantQuery,
{
    #[tracing::instrument(
        name = "CreateUserPermissionGrantUseCase::create_user_permission_grant",
        skip(self)
    )]
    async fn create_user_permission_grant(
        &self,
        req: &CreateUserPermission,
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
            .insert_user_permission_grant(&CreateUserPermissionGrant {
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
        inserts: Vec<(Uuid, Uuid)>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn snapshot(log: &SharedLog) -> (Vec<Uuid>, Vec<String>, Vec<(Uuid, Uuid)>) {
        let log = log.lock().expect("log");
        (
            log.user_lookups.clone(),
            log.permission_lookups.clone(),
            log.inserts.clone(),
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
    impl InsertUserPermissionGrantQuery for MockGrants {
        async fn insert_user_permission_grant(
            &self,
            req: &CreateUserPermissionGrant,
        ) -> Result<UserPermissionGrant, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .inserts
                .push((req.user_id, req.permission_id));

            if self.fail {
                return Err("simulated grant insert failure".into());
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
    ) -> CreateUserPermissionGrantUseCase<MockUsers, MockPermissions, MockGrants> {
        CreateUserPermissionGrantUseCase::new(
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

    fn base_request() -> CreateUserPermission {
        CreateUserPermission {
            user_id: user_id(),
            permission: "oxidauth:users:read".to_owned(),
        }
    }

    #[tokio::test]
    async fn user_is_validated_permission_is_resolved_by_parts_then_inserted_by_id() {
        let log = Arc::new(Mutex::new(Log::default()));

        let result = use_case(&log, false, false, false)
            .create_user_permission_grant(&base_request())
            .await
            .expect("grant succeeds");

        let (users, permissions, inserts) = snapshot(&log);
        assert_eq!(users, vec![user_id()]);
        assert_eq!(permissions, vec!["oxidauth:users:read".to_owned()]);
        assert_eq!(
            inserts,
            vec![(user_id(), permission_id())],
            "the grant stores the resolved permission id, not the string"
        );
        assert_eq!(result.permission.to_string(), "oxidauth:users:read");
        assert_eq!(result.grant.user_id, user_id());
    }

    #[tokio::test]
    async fn an_unresolvable_permission_aborts_before_the_grant_insert() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, false, true, false)
            .create_user_permission_grant(&base_request())
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
            .create_user_permission_grant(&base_request())
            .await
            .expect_err("user lookup failure propagates");

        assert_eq!(error.to_string(), "simulated user lookup failure");
        let (_, permissions, inserts) = snapshot(&log);
        assert!(permissions.is_empty());
        assert!(inserts.is_empty());
    }

    #[tokio::test]
    async fn grant_insert_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, false, false, true)
            .create_user_permission_grant(&base_request())
            .await
            .expect_err("grant insert failure propagates");

        assert_eq!(error.to_string(), "simulated grant insert failure");
        assert_eq!(snapshot(&log).2.len(), 1);
    }
}
