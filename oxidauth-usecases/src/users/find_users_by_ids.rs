use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    users::find_users_by_ids::{FindUsersByIds, FindUsersByIdsTrait, UsersByIds},
};
use oxidauth_repository::users::select_users_by_ids_query::SelectUsersByIdsQuery;

pub struct FindUsersByIdsUseCase<T>
where
    T: SelectUsersByIdsQuery,
{
    users: T,
}

impl<T> FindUsersByIdsUseCase<T>
where
    T: SelectUsersByIdsQuery,
{
    pub fn new(users: T) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<T> FindUsersByIdsTrait for FindUsersByIdsUseCase<T>
where
    T: SelectUsersByIdsQuery,
{
    #[tracing::instrument(name = "find_users_by_ids_usecase", skip(self))]
    async fn find_users_by_ids(
        &self,
        params: &FindUsersByIds,
    ) -> Result<UsersByIds, BoxedError> {
        self.users.call(params).await
    }
}
