use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, roles::create_role::*};
use oxidauth_repository::roles::insert_role::InsertRoleQuery;

pub struct CreateRoleUseCase<T>
where
    T: InsertRoleQuery,
{
    roles: T,
}

impl<T> CreateRoleUseCase<T>
where
    T: InsertRoleQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<T> CreateRoleServiceTrait for CreateRoleUseCase<T>
where
    T: InsertRoleQuery,
{
    #[tracing::instrument(name = "CreateRoleUseCase::create_role", skip(self))]
    async fn create_role(&self, req: &CreateRole) -> Result<Role, BoxedError> {
        self.roles
            .insert_role(req)
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
            id: uuid::Uuid::new_v4(),
            name: "admin".to_owned(),
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

    struct MockRoles {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl InsertRoleQuery for MockRoles {
        async fn insert_role(&self, req: &CreateRole) -> Result<Role, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .inserts
                .push(req.name.clone());

            if self.fail {
                return Err("simulated insert failure".into());
            }

            Ok(stored_role())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> CreateRoleUseCase<MockRoles> {
        CreateRoleUseCase::new(MockRoles {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate — verbatim passthrough + error propagation is the
    // whole contract.
    #[tokio::test]
    async fn name_reaches_the_insert_query_verbatim_and_the_row_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));

        let role = use_case(&log, false)
            .create_role(&CreateRole {
                name: "admin".to_owned(),
            })
            .await
            .expect("create succeeds");

        assert_eq!(inserts(&log), vec!["admin".to_owned()]);
        assert_eq!(role.name, "admin");
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .create_role(&CreateRole {
                name: "admin".to_owned(),
            })
            .await
            .expect_err("duplicate-name insert failure propagates");

        assert_eq!(error.to_string(), "simulated insert failure");
    }
}
