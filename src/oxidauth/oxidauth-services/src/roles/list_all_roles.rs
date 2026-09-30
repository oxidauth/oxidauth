use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, roles::list_all_roles::*};
use oxidauth_repository::roles::select_all_roles::SelectAllRolesQuery;

pub struct ListAllRolesUseCase<T>
where
    T: SelectAllRolesQuery,
{
    roles: T,
}

impl<T> ListAllRolesUseCase<T>
where
    T: SelectAllRolesQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<T> ListAllRolesServiceTrait for ListAllRolesUseCase<T>
where
    T: SelectAllRolesQuery,
{
    #[tracing::instrument(name = "ListAllRolesUseCase::list_all_roles", skip(self))]
    async fn list_all_roles(&self, req: &ListAllRoles) -> Result<Vec<Role>, BoxedError> {
        self.roles
            .select_all_roles(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;

    use super::*;

    fn stored_roles() -> Vec<Role> {
        ["admin", "editor"]
            .iter()
            .map(|name| {
                Role {
                    id: uuid::uuid!("11111111-1111-4111-8111-111111111111"),
                    name: (*name).to_owned(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }
            })
            .collect()
    }

    #[derive(Default)]
    struct Log {
        calls: usize,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn calls(log: &SharedLog) -> usize {
        log.lock().expect("log").calls
    }

    struct MockRoles {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl SelectAllRolesQuery for MockRoles {
        async fn select_all_roles(&self, params: &ListAllRoles) -> Result<Vec<Role>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .calls += 1;

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(stored_roles())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> ListAllRolesUseCase<MockRoles> {
        ListAllRolesUseCase::new(MockRoles {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate: query order passes through with no re-sort/filter.
    #[tokio::test]
    async fn the_stored_list_is_returned_in_query_order() {
        let log = Arc::new(Mutex::new(Log::default()));

        let roles = use_case(&log, false)
            .list_all_roles(&ListAllRoles)
            .await
            .expect("list succeeds");

        assert_eq!(calls(&log), 1);
        assert_eq!(
            roles
                .iter()
                .map(|r| r.name.as_str())
                .collect::<Vec<_>>(),
            vec!["admin", "editor"]
        );
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .list_all_roles(&ListAllRoles)
            .await
            .expect_err("select failure propagates");

        assert_eq!(error.to_string(), "simulated select failure");
    }
}
