use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, service::Service, users::find_users_by_ids::*,
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
impl<'a, T> Service<&'a FindUsersByIds> for FindUsersByIdsUseCase<T>
where
    T: SelectUsersByIdsQuery,
{
    type Response = UsersByIds;
    type Error = BoxedError;

    #[tracing::instrument(name = "find_users_by_ids_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a FindUsersByIds,
    ) -> Result<Self::Response, Self::Error> {
        self.users.call(req).await
    }
}
