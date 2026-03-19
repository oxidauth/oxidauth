use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    users::list_all_users::{ListAllUsers, ListAllUsersTrait, User},
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
impl<T> ListAllUsersTrait for ListAllUsersUseCase<T>
where
    T: SelectAllUsersQuery,
{
    #[tracing::instrument(name = "list_all_users_usecase", skip(self))]
    async fn list_all_users(
        &self,
        params: &ListAllUsers,
    ) -> Result<Vec<User>, BoxedError> {
        self.users.call(params).await
    }
}
