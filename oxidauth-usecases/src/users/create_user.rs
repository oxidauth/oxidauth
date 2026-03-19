use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    users::create_user::{CreateUser, CreateUserTrait, User},
};
use oxidauth_repository::users::insert_user::InsertUserQuery;

#[derive(Clone)]
pub struct CreateUserUseCase<T>
where
    T: InsertUserQuery,
{
    users: T,
}

impl<T> CreateUserUseCase<T>
where
    T: InsertUserQuery,
{
    pub fn new(users: T) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<T> CreateUserTrait for CreateUserUseCase<T>
where
    T: InsertUserQuery,
{
    #[tracing::instrument(name = "create_user_usecase", skip(self))]
    async fn create_user(
        &self,
        params: &CreateUser,
    ) -> Result<User, BoxedError> {
        let user = self.users.call(params).await?;

        Ok(user)
    }
}
