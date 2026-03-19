use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    users::{
        find_user_by_username::{FindUserByUsername, FindUserByUsernameTrait, User},
        UserNotFoundError,
    },
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
impl<T> FindUserByUsernameTrait for FindUserByUsernameUseCase<T>
where
    T: SelectUserByUsernameQuery,
{
    #[tracing::instrument(name = "find_user_by_username_usecase", skip(self))]
    async fn find_user_by_username(
        &self,
        params: &FindUserByUsername,
    ) -> Result<User, BoxedError> {
        let user = self
            .users
            .call(&params.username)
            .await?
            .ok_or_else(|| UserNotFoundError::username(&params.username))?;

        Ok(user)
    }
}
