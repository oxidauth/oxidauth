use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    service::Service,
    users::{find_user_by_username::*, UserNotFoundError},
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
impl<'a, T> Service<&'a FindUserByUsername> for FindUserByUsernameUseCase<T>
where
    T: SelectUserByUsernameQuery,
{
    type Response = User;
    type Error = BoxedError;

    #[tracing::instrument(name = "find_user_by_username_usecase", skip(self))]
    async fn call(
        &self,
        params: &'a FindUserByUsername,
    ) -> Result<Self::Response, Self::Error> {
        let user = self
            .users
            .call(&params.username)
            .await?
            .ok_or_else(|| UserNotFoundError::username(&params.username))?;

        Ok(user)
    }
}
