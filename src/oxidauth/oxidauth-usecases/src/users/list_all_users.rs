use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, service::Service, users::list_all_users::*,
};
use oxidauth_repository::users::select_all_users_query::SelectAllUsersQuery;

pub struct ListAllUsersUseCase<T>
where
    T: SelectAllUsersQuery,
{
    users: T,
}

impl<T> ListAllUsersUseCase<T>
where
    T: SelectAllUsersQuery,
{
    pub fn new(users: T) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<'a, T> Service<&'a ListAllUsers> for ListAllUsersUseCase<T>
where
    T: SelectAllUsersQuery,
{
    type Response = Vec<User>;
    type Error = BoxedError;

    #[tracing::instrument(name = "list_all_users_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a ListAllUsers,
    ) -> Result<Self::Response, Self::Error> {
        self.users.call(req).await
    }
}
