use async_trait::async_trait;
use std::sync::Arc;

use oxidauth_kernel::{users::create_user::*, error::BoxedError};
use oxidauth_repository::users::insert_user::InsertUserQuery;

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
impl<'a, T> Service<&'a CreateUser> for CreateUserUseCase<T>
where
    T: InsertUserQuery,
{
    type Response = User;
    type Error = BoxedError;

    #[tracing::instrument(name = "create_user_usecase", skip(self))]
    async fn call(&self, req: &'a CreateUser) -> Result<Self::Response, Self::Error> {
        self.users
            .call(req)
            .await
    }
}
