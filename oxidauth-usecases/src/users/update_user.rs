use async_trait::async_trait;

use oxidauth_kernel::{error::BoxedError, users::update_user::*};
use oxidauth_repository::users::update_user::UpdateUserQuery;

pub struct UpdateUserUseCase<T>
where
    T: UpdateUserQuery,
{
    users: T,
}

impl<T> UpdateUserUseCase<T>
where
    T: UpdateUserQuery,
{
    pub fn new(users: T) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<'a, T> Service<&'a UpdateUser> for UpdateUserUseCase<T>
where
    T: UpdateUserQuery,
{
    type Response = User;
    type Error = BoxedError;

    #[tracing::instrument(name = "update_user_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a UpdateUser,
    ) -> Result<Self::Response, Self::Error> {
        self.users.call(req).await
    }
}
