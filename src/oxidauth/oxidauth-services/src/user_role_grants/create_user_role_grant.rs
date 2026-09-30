use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    roles::find_role_by_id::FindRoleById,
    user_role_grants::{UserRole, create_user_role_grant::*},
    users::find_user_by_id::FindUserById,
};
use oxidauth_repository::{
    roles::select_role_by_id::SelectRoleByIdQuery,
    user_role_grants::insert_user_role_grant::InsertUserRoleGrantQuery,
    users::select_user_by_id_query::SelectUserByIdQuery,
};

pub struct CreateUserRoleGrantUseCase<U, R, UR>
where
    U: SelectUserByIdQuery,
    R: SelectRoleByIdQuery,
    UR: InsertUserRoleGrantQuery,
{
    users: U,
    roles: R,
    user_role_grants: UR,
}

impl<U, R, UR> CreateUserRoleGrantUseCase<U, R, UR>
where
    U: SelectUserByIdQuery,
    R: SelectRoleByIdQuery,
    UR: InsertUserRoleGrantQuery,
{
    pub fn new(users: U, roles: R, user_role_grants: UR) -> Self {
        Self {
            users,
            roles,
            user_role_grants,
        }
    }
}

#[async_trait]
impl<U, R, UR> CreateUserRoleGrantServiceTrait for CreateUserRoleGrantUseCase<U, R, UR>
where
    U: SelectUserByIdQuery,
    R: SelectRoleByIdQuery,
    UR: InsertUserRoleGrantQuery,
{
    #[tracing::instrument(
        name = "CreateUserRoleGrantUseCase::create_user_role_grant",
        skip(self)
    )]
    async fn create_user_role_grant(
        &self,
        req: &CreateUserRoleGrant,
    ) -> Result<UserRole, BoxedError> {
        let user = self
            .users
            .select_user_by_id(&FindUserById {
                user_id: req.user_id,
            })
            .await?;

        let role = self
            .roles
            .select_role_by_id(&FindRoleById {
                role_id: req.role_id,
            })
            .await?;

        let grant = self
            .user_role_grants
            .insert_user_role_grant(&CreateUserRoleGrant {
                user_id: user.id,
                role_id: role.id,
            })
            .await?;

        Ok(UserRole { role, grant })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use oxidauth_kernel::{
        roles::Role,
        user_role_grants::UserRoleGrant,
        users::{User, UserKind, UserStatus},
    };
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn user_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    fn role_id() -> Uuid {
        uuid::uuid!("22222222-2222-4222-8222-222222222222")
    }

    fn stored_user() -> User {
        User {
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
        }
    }

    fn stored_role() -> Role {
        Role {
            id: role_id(),
            name: "admin".to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn stored_grant() -> UserRoleGrant {
        UserRoleGrant {
            user_id: user_id(),
            role_id: role_id(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Default)]
    struct Log {
        user_lookups: Vec<Uuid>,
        role_lookups: Vec<Uuid>,
        grants: Vec<(Uuid, Uuid)>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn snapshot(log: &SharedLog) -> (Vec<Uuid>, Vec<Uuid>, Vec<(Uuid, Uuid)>) {
        let log = log.lock().expect("log");
        (
            log.user_lookups.clone(),
            log.role_lookups.clone(),
            log.grants.clone(),
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

            Ok(stored_user())
        }
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

    struct MockGrants {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl InsertUserRoleGrantQuery for MockGrants {
        async fn insert_user_role_grant(
            &self,
            req: &CreateUserRoleGrant,
        ) -> Result<UserRoleGrant, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .grants
                .push((req.user_id, req.role_id));

            if self.fail {
                return Err("simulated grant insert failure".into());
            }

            Ok(stored_grant())
        }
    }

    fn use_case(
        log: &SharedLog,
        user_fails: bool,
        role_fails: bool,
        grant_fails: bool,
    ) -> CreateUserRoleGrantUseCase<MockUsers, MockRoles, MockGrants> {
        CreateUserRoleGrantUseCase::new(
            MockUsers {
                log: log.clone(),
                fail: user_fails,
            },
            MockRoles {
                log: log.clone(),
                fail: role_fails,
            },
            MockGrants {
                log: log.clone(),
                fail: grant_fails,
            },
        )
    }

    fn base_request() -> CreateUserRoleGrant {
        CreateUserRoleGrant {
            user_id: user_id(),
            role_id: role_id(),
        }
    }

    #[tokio::test]
    async fn both_rows_are_validated_then_the_grant_is_assembled() {
        let log = Arc::new(Mutex::new(Log::default()));

        let result = use_case(&log, false, false, false)
            .create_user_role_grant(&base_request())
            .await
            .expect("grant succeeds");

        let (users, roles, grants) = snapshot(&log);
        assert_eq!(users, vec![user_id()]);
        assert_eq!(roles, vec![role_id()]);
        assert_eq!(grants, vec![(user_id(), role_id())]);
        assert_eq!(result.role.name, "admin");
        assert_eq!(result.grant.user_id, user_id());
        assert_eq!(result.grant.role_id, role_id());
    }

    #[tokio::test]
    async fn a_missing_user_aborts_before_role_lookup_or_grant_insert() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true, false, false)
            .create_user_role_grant(&base_request())
            .await
            .expect_err("user lookup failure propagates");

        assert_eq!(error.to_string(), "simulated user lookup failure");
        let (_, roles, grants) = snapshot(&log);
        assert!(roles.is_empty(), "the role must not be looked up");
        assert!(grants.is_empty(), "no grant row after a failed find");
    }

    #[tokio::test]
    async fn a_missing_role_aborts_before_the_grant_insert() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, false, true, false)
            .create_user_role_grant(&base_request())
            .await
            .expect_err("role lookup failure propagates");

        assert_eq!(error.to_string(), "simulated role lookup failure");
        assert!(
            snapshot(&log).2.is_empty(),
            "no grant row after a failed find"
        );
    }

    #[tokio::test]
    async fn grant_insert_error_surfaces_unmapped() {
        // duplicate pair / invalid fk errors come from the database
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, false, false, true)
            .create_user_role_grant(&base_request())
            .await
            .expect_err("grant insert failure propagates");

        assert_eq!(error.to_string(), "simulated grant insert failure");
        assert_eq!(snapshot(&log).2.len(), 1);
    }
}
