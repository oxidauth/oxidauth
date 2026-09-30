use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    role_role_grants::create_role_role_grant::*,
    roles::find_role_by_id::FindRoleById,
};
use oxidauth_repository::{
    role_role_grants::insert_role_role_grant::InsertRoleRoleGrantQuery,
    roles::select_role_by_id::SelectRoleByIdQuery,
};

pub struct CreateRoleRoleGrantUseCase<T, R>
where
    T: InsertRoleRoleGrantQuery,
    R: SelectRoleByIdQuery,
{
    role_role_grants: T,
    roles: R,
}

impl<T, R> CreateRoleRoleGrantUseCase<T, R>
where
    T: InsertRoleRoleGrantQuery,
    R: SelectRoleByIdQuery,
{
    pub fn new(role_role_grants: T, roles: R) -> Self {
        Self {
            role_role_grants,
            roles,
        }
    }
}

#[async_trait]
impl<T, R> CreateRoleRoleGrantServiceTrait for CreateRoleRoleGrantUseCase<T, R>
where
    T: InsertRoleRoleGrantQuery,
    R: SelectRoleByIdQuery,
{
    #[tracing::instrument(
        name = "CreateRoleRoleGrantUseCase::create_role_role_grant",
        skip(self)
    )]
    async fn create_role_role_grant(
        &self,
        req: &CreateRoleRoleGrant,
    ) -> Result<RoleRoleGrantDetail, BoxedError> {
        self.roles
            .select_role_by_id(&FindRoleById {
                role_id: req.parent_id,
            })
            .await?;

        let child = self
            .roles
            .select_role_by_id(&FindRoleById {
                role_id: req.child_id,
            })
            .await?;

        let grant = self
            .role_role_grants
            .insert_role_role_grant(req)
            .await?;

        Ok(RoleRoleGrantDetail { role: child, grant })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::{role_role_grants::RoleRoleGrant, roles::Role};
    use uuid::Uuid;

    use super::*;

    fn parent_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    fn child_id() -> Uuid {
        uuid::uuid!("22222222-2222-4222-8222-222222222222")
    }

    fn role(id: Uuid) -> Role {
        Role {
            id,
            name: if id == parent_id() {
                "parent"
            } else {
                "child"
            }
            .to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn stored_grant() -> RoleRoleGrant {
        RoleRoleGrant {
            parent_id: parent_id(),
            child_id: child_id(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Default)]
    struct Log {
        role_lookups: Vec<Uuid>,
        inserts: Vec<(Uuid, Uuid)>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn snapshot(log: &SharedLog) -> (Vec<Uuid>, Vec<(Uuid, Uuid)>) {
        let log = log.lock().expect("log");
        (log.role_lookups.clone(), log.inserts.clone())
    }

    /// Errors once for whichever id `breaks` names; parent is looked up
    /// first, child second.
    struct MockRoles {
        log: SharedLog,
        breaks: Option<Uuid>,
    }

    #[async_trait]
    impl SelectRoleByIdQuery for MockRoles {
        async fn select_role_by_id(&self, req: &FindRoleById) -> Result<Role, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .role_lookups
                .push(req.role_id);

            if self.breaks == Some(req.role_id) {
                return Err("simulated role lookup failure".into());
            }

            Ok(role(req.role_id))
        }
    }

    struct MockGrants {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl InsertRoleRoleGrantQuery for MockGrants {
        async fn insert_role_role_grant(
            &self,
            req: &CreateRoleRoleGrant,
        ) -> Result<RoleRoleGrant, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .inserts
                .push((req.parent_id, req.child_id));

            if self.fail {
                return Err("simulated grant insert failure".into());
            }

            Ok(stored_grant())
        }
    }

    fn use_case(
        log: &SharedLog,
        breaks: Option<Uuid>,
        grant_fails: bool,
    ) -> CreateRoleRoleGrantUseCase<MockGrants, MockRoles> {
        CreateRoleRoleGrantUseCase::new(
            MockGrants {
                log: log.clone(),
                fail: grant_fails,
            },
            MockRoles {
                log: log.clone(),
                breaks,
            },
        )
    }

    fn base_request() -> CreateRoleRoleGrant {
        CreateRoleRoleGrant {
            parent_id: parent_id(),
            child_id: child_id(),
        }
    }

    #[tokio::test]
    async fn parent_is_validated_but_the_detail_carries_the_child_role() {
        let log = Arc::new(Mutex::new(Log::default()));

        let detail = use_case(&log, None, false)
            .create_role_role_grant(&base_request())
            .await
            .expect("grant succeeds");

        let (lookups, inserts) = snapshot(&log);
        assert_eq!(
            lookups,
            vec![parent_id(), child_id()],
            "both endpoints are validated before the insert"
        );
        assert_eq!(inserts, vec![(parent_id(), child_id())]);
        assert_eq!(
            detail.role.name, "child",
            "the returned detail is the child role; the parent row is only
             an existence check"
        );
        assert_eq!(detail.grant.parent_id, parent_id());
    }

    #[tokio::test]
    async fn an_unknown_parent_aborts_before_the_child_lookup() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, Some(parent_id()), false)
            .create_role_role_grant(&base_request())
            .await
            .expect_err("parent lookup failure propagates");

        assert_eq!(error.to_string(), "simulated role lookup failure");
        let (lookups, inserts) = snapshot(&log);
        assert_eq!(lookups, vec![parent_id()], "the child is never resolved");
        assert!(inserts.is_empty());
    }

    #[tokio::test]
    async fn an_unknown_child_aborts_before_the_grant_insert() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, Some(child_id()), false)
            .create_role_role_grant(&base_request())
            .await
            .expect_err("child lookup failure propagates");

        assert_eq!(error.to_string(), "simulated role lookup failure");
        assert!(snapshot(&log).1.is_empty());
    }

    #[tokio::test]
    async fn grant_insert_error_surfaces_unmapped() {
        // cycle / self-reference guards live in the SQL layer (A6) — raw
        // errors must reach callers
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, None, true)
            .create_role_role_grant(&base_request())
            .await
            .expect_err("grant insert failure propagates");

        assert_eq!(error.to_string(), "simulated grant insert failure");
        assert_eq!(snapshot(&log).1.len(), 1);
    }
}
