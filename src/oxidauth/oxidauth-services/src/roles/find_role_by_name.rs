use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, roles::find_role_by_name::*};
use oxidauth_repository::roles::select_role_by_name::SelectRoleByNameQuery;

pub struct FindRoleByNameUseCase<T>
where
    T: SelectRoleByNameQuery,
{
    roles: T,
}

impl<T> FindRoleByNameUseCase<T>
where
    T: SelectRoleByNameQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<T> FindRoleByNameServiceTrait for FindRoleByNameUseCase<T>
where
    T: SelectRoleByNameQuery,
{
    #[tracing::instrument(name = "FindRoleByNameUseCase::find_role_by_name", skip(self))]
    async fn find_role_by_name(&self, req: &FindRoleByName) -> Result<Role, BoxedError> {
        self.roles
            .select_role_by_name(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;

    use super::*;

    fn stored_role() -> Role {
        Role {
            id: uuid::uuid!("11111111-1111-4111-8111-111111111111"),
            name: "admin".to_owned(),
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

    struct MockRoles {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl SelectRoleByNameQuery for MockRoles {
        async fn select_role_by_name(&self, req: &FindRoleByName) -> Result<Role, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .lookups
                .push(req.role.clone());

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(stored_role())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> FindRoleByNameUseCase<MockRoles> {
        FindRoleByNameUseCase::new(MockRoles {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate.
    #[tokio::test]
    async fn name_reaches_the_query_and_the_row_is_returned_verbatim() {
        let log = Arc::new(Mutex::new(Log::default()));

        let role = use_case(&log, false)
            .find_role_by_name(&FindRoleByName {
                role: "admin".to_owned(),
            })
            .await
            .expect("find succeeds");

        assert_eq!(lookups(&log), vec!["admin".to_owned()]);
        assert_eq!(role.name, "admin");
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .find_role_by_name(&FindRoleByName {
                role: "ghost".to_owned(),
            })
            .await
            .expect_err("missing-role error propagates unchanged");

        assert_eq!(error.to_string(), "simulated select failure");
    }
}
