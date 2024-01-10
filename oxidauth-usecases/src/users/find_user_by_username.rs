use async_trait::async_trait;
use uuid::Uuid;

use oxidauth_kernel::{
    error::BoxedError, service::Service, users::find_user_by_username::*,
};
use oxidauth_repository::users::select_user_by_username_query::SelectUserByUsernameQuery;

pub struct FindUserByUsernameUseCase<T>
where
    T: SelectUserByUsernameQuery,
{
    users: T,
}

impl<T> FindUserByUsernameUseCase<T>
where
    T: SelectUserByUsernameQuery,
{
    pub fn new(users: T) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<T> Service<String> for FindUserByUsernameUseCase<T>
where
    T: SelectUserByUsernameQuery,
{
    type Response = User;
    type Error = BoxedError;

    #[tracing::instrument(name = "find_user_by_username_usecase", skip(self))]
    async fn call(&self, req: String) -> Result<Self::Response, Self::Error> {
        self.users.call(req).await
    }
}
