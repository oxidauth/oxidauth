use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, roles::update_role::*};
use oxidauth_repository::roles::update_role::UpdateRoleQuery;

pub struct UpdateRoleUseCase<T>
where
    T: UpdateRoleQuery,
{
    roles: T,
}

impl<T> UpdateRoleUseCase<T>
where
    T: UpdateRoleQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<T> UpdateRoleServiceTrait for UpdateRoleUseCase<T>
where
    T: UpdateRoleQuery,
{
    #[tracing::instrument(name = "UpdateRoleUseCase::update_role", skip(self))]
    async fn update_role(&self, req: &UpdateRole) -> Result<Role, BoxedError> {
        self.roles
            .update_role(req)
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

    fn stored_role() -> Role {
        Role {
            id: role_id(),
            name: "renamed".to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Default)]
    struct Log {
        updates: Vec<(Option<Uuid>, String)>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn updates(log: &SharedLog) -> Vec<(Option<Uuid>, String)> {
        log.lock()
            .expect("log")
            .updates
            .clone()
    }

    struct MockRoles {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl UpdateRoleQuery for MockRoles {
        async fn update_role(&self, req: &UpdateRole) -> Result<Role, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .updates
                .push((req.role_id, req.name.clone()));

            if self.fail {
                return Err("simulated update failure".into());
            }

            Ok(stored_role())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> UpdateRoleUseCase<MockRoles> {
        UpdateRoleUseCase::new(MockRoles {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate (unlike `users::update_user`, no current-row backfill:
    // even `role_id: None` goes straight to the query for the repository
    // to resolve).
    #[tokio::test]
    async fn request_reaches_the_query_verbatim_including_a_missing_role_id() {
        let log = Arc::new(Mutex::new(Log::default()));

        let role = use_case(&log, false)
            .update_role(&UpdateRole {
                role_id: None,
                name: "renamed".to_owned(),
            })
            .await
            .expect("update succeeds");

        assert_eq!(
            updates(&log),
            vec![(None, "renamed".to_owned())],
            "the service must not resolve or reject the missing id itself"
        );
        assert_eq!(role.name, "renamed");
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .update_role(&UpdateRole {
                role_id: Some(role_id()),
                name: "renamed".to_owned(),
            })
            .await
            .expect_err("update failure propagates");

        assert_eq!(error.to_string(), "simulated update failure");
    }
}
