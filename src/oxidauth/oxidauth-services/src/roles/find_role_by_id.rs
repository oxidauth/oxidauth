use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, roles::find_role_by_id::*};
use oxidauth_repository::roles::select_role_by_id::SelectRoleByIdQuery;

pub struct FindRoleByIdUseCase<T>
where
    T: SelectRoleByIdQuery,
{
    roles: T,
}

impl<T> FindRoleByIdUseCase<T>
where
    T: SelectRoleByIdQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<T> FindRoleByIdServiceTrait for FindRoleByIdUseCase<T>
where
    T: SelectRoleByIdQuery,
{
    #[tracing::instrument(name = "FindRoleByIdUseCase::find_role_by_id", skip(self))]
    async fn find_role_by_id(&self, req: &FindRoleById) -> Result<Role, BoxedError> {
        self.roles
            .select_role_by_id(req)
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
            name: "admin".to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Default)]
    struct Log {
        calls: Vec<Uuid>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn calls(log: &SharedLog) -> Vec<Uuid> {
        log.lock()
            .expect("log")
            .calls
            .clone()
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
                .calls
                .push(req.role_id);

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(stored_role())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> FindRoleByIdUseCase<MockRoles> {
        FindRoleByIdUseCase::new(MockRoles {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate: missing rows raise `RowNotFound` inside the query, so
    // the service only ever relays a row or an error.
    #[tokio::test]
    async fn id_reaches_the_query_and_the_row_is_returned_verbatim() {
        let log = Arc::new(Mutex::new(Log::default()));

        let role = use_case(&log, false)
            .find_role_by_id(&FindRoleById { role_id: role_id() })
            .await
            .expect("find succeeds");

        assert_eq!(calls(&log), vec![role_id()]);
        assert_eq!(role.id, role_id());
        assert_eq!(role.name, "admin");
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .find_role_by_id(&FindRoleById { role_id: role_id() })
            .await
            .expect_err("RowNotFound from the query propagates unchanged");

        assert_eq!(error.to_string(), "simulated select failure");
    }
}
