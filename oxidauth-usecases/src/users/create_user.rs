use async_trait::async_trait;
use std::sync::Arc;

use oxidauth_kernel::{users::create_user::*, error::BoxedError};
use oxidauth_repository::users::insert_user::InsertUserQuery;

pub struct CreateUserUseCase {
    users: InsertUserQuery,
}

impl CreateUserUseCase {
    pub fn new(users: impl InsertUserQuery) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<'a> Service<&'a CreateUser> for CreateUserUseCase
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
