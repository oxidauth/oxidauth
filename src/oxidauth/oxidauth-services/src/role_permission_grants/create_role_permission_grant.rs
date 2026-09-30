use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    permissions::{PermissionNotFoundError, find_permission_by_parts::FindPermissionByParts},
    role_permission_grants::create_role_permission_grant::*,
    roles::find_role_by_id::FindRoleById,
};
use oxidauth_repository::{
    permissions::select_permission_by_parts::SelectPermissionByPartsQuery,
    role_permission_grants::insert_role_permission_grant::*,
    roles::select_role_by_id::SelectRoleByIdQuery,
};

pub struct CreateRolePermissionGrantUseCase<T, R, P>
where
    T: InsertRolePermissionGrantQuery,
    R: SelectRoleByIdQuery,
    P: SelectPermissionByPartsQuery,
{
    role_permission_grants: T,
    roles: R,
    permissions: P,
}

impl<T, R, P> CreateRolePermissionGrantUseCase<T, R, P>
where
    T: InsertRolePermissionGrantQuery,
    R: SelectRoleByIdQuery,
    P: SelectPermissionByPartsQuery,
{
    pub fn new(role_permission_grants: T, roles: R, permissions: P) -> Self {
        Self {
            role_permission_grants,
            roles,
            permissions,
        }
    }
}

#[async_trait]
impl<T, R, P> CreateRolePermissionGrantServiceTrait for CreateRolePermissionGrantUseCase<T, R, P>
where
    T: InsertRolePermissionGrantQuery,
    R: SelectRoleByIdQuery,
    P: SelectPermissionByPartsQuery,
{
    #[tracing::instrument(
        name = "CreateRolePermissionGrantUseCase::create_role_permission_grant",
        skip(self)
    )]
    async fn create_role_permission_grant(
        &self,
        req: &CreateRolePermissionGrant,
    ) -> Result<RolePermission, BoxedError> {
        self.roles
            .select_role_by_id(&FindRoleById {
                role_id: req.role_id,
            })
            .await?;

        let permission = self
            .permissions
            .select_permission_by_parts(&FindPermissionByParts {
                permission: req.permission.to_owned(),
            })
            .await?
            .ok_or_else(|| PermissionNotFoundError::new(&req.permission))?;

        let grant = self
            .role_permission_grants
            .insert_role_permission_grant(&InsertRolePermissionGrant {
                role_id: req.role_id,
                permission_id: permission.id,
            })
            .await?;

        Ok(RolePermission { permission, grant })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::{
        permissions::Permission,
        role_permission_grants::RolePermissionGrant,
        roles::Role,
    };
    use uuid::Uuid;

    use super::*;

    fn role_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    fn permission_id() -> Uuid {
        uuid::uuid!("33333333-3333-4333-8333-333333333333")
    }

    fn stored_role() -> Role {
        Role {
            id: role_id(),
            name: "admin".to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn stored_permission() -> Permission {
        Permission {
            id: permission_id(),
            realm: "oxidauth".to_owned(),
            resource: "users".to_owned(),
            action: "read".to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Default)]
    struct Log {
        role_lookups: Vec<Uuid>,
        permission_lookups: Vec<String>,
        inserts: Vec<(Uuid, Uuid)>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn snapshot(log: &SharedLog) -> (Vec<Uuid>, Vec<String>, Vec<(Uuid, Uuid)>) {
        let log = log.lock().expect("log");
        (
            log.role_lookups.clone(),
            log.permission_lookups.clone(),
            log.inserts.clone(),
        )
    }

    struct MockRoles {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl SelectRoleByIdQuery for MockRoles {
        async fn select_role_by_id(&self, req: &FindRoleById) -> Result<Role, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .role_lookups
                .push(req.role_id);

            if self.fail {
                return Err("simulated role lookup failure".into());
            }

            Ok(stored_role())
        }
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
                .permission_lookups
                .push(req.permission.clone());

            if self.fail {
                return Err("simulated permission lookup failure".into());
            }

            if self.missing {
                return Ok(None);
            }

            Ok(Some(stored_permission()))
        }
    }

    struct MockGrants {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl InsertRolePermissionGrantQuery for MockGrants {
        async fn insert_role_permission_grant(
            &self,
            req: &InsertRolePermissionGrant,
        ) -> Result<RolePermissionGrant, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .inserts
                .push((req.role_id, req.permission_id));

            if self.fail {
                return Err("simulated grant insert failure".into());
            }

            Ok(RolePermissionGrant {
                role_id: req.role_id,
                permission_id: req.permission_id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    fn use_case(
        log: &SharedLog,
        role_fails: bool,
        permission_missing: bool,
        permission_fails: bool,
        grant_fails: bool,
    ) -> CreateRolePermissionGrantUseCase<MockGrants, MockRoles, MockPermissions> {
        CreateRolePermissionGrantUseCase::new(
            MockGrants {
                log: log.clone(),
                fail: grant_fails,
            },
            MockRoles {
                log: log.clone(),
                fail: role_fails,
            },
            MockPermissions {
                log: log.clone(),
                missing: permission_missing,
                fail: permission_fails,
            },
        )
    }

    fn base_request() -> CreateRolePermissionGrant {
        CreateRolePermissionGrant {
            role_id: role_id(),
            permission: "oxidauth:users:read".to_owned(),
        }
    }

    #[tokio::test]
    async fn role_is_validated_permission_is_resolved_by_parts_then_inserted_by_id() {
        let log = Arc::new(Mutex::new(Log::default()));

        let result = use_case(&log, false, false, false, false)
            .create_role_permission_grant(&base_request())
            .await
            .expect("grant succeeds");

        let (roles, permissions, inserts) = snapshot(&log);
        assert_eq!(roles, vec![role_id()]);
        assert_eq!(permissions, vec!["oxidauth:users:read".to_owned()]);
        assert_eq!(
            inserts,
            vec![(role_id(), permission_id())],
            "the grant stores the resolved permission id, not the string"
        );
        assert_eq!(result.permission.to_string(), "oxidauth:users:read");
        assert_eq!(result.grant.role_id, role_id());
    }

    #[tokio::test]
    async fn an_unresolvable_permission_aborts_before_the_grant_insert() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, false, true, false, false)
            .create_role_permission_grant(&base_request())
            .await
            .expect_err("Ok(None) from the parts lookup becomes not-found");

        assert_eq!(
            error.to_string(),
            "permission with name not found: oxidauth:users:read"
        );
        assert!(snapshot(&log).2.is_empty());
    }

    #[tokio::test]
    async fn a_missing_role_aborts_before_the_permission_lookup() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true, false, false, false)
            .create_role_permission_grant(&base_request())
            .await
            .expect_err("role lookup failure propagates");

        assert_eq!(error.to_string(), "simulated role lookup failure");
        let (_, permissions, inserts) = snapshot(&log);
        assert!(permissions.is_empty());
        assert!(inserts.is_empty());
    }

    #[tokio::test]
    async fn grant_insert_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, false, false, false, true)
            .create_role_permission_grant(&base_request())
            .await
            .expect_err("grant insert failure propagates");

        assert_eq!(error.to_string(), "simulated grant insert failure");
        assert_eq!(snapshot(&log).2.len(), 1);
    }
}
