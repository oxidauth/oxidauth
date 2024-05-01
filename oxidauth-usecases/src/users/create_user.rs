use async_trait::async_trait;

use oxidauth_kernel::auth_keys::create_auth_key::{
    CreateAuthKey, CreateAuthKeyService,
};
use oxidauth_kernel::{error::BoxedError, users::create_user::*};
use oxidauth_repository::users::insert_user::InsertUserQuery;

pub struct CreateUserUseCase<T>
where
    T: InsertUserQuery,
{
    users: T,
    auth_keys: CreateAuthKeyService,
}

impl<T> CreateUserUseCase<T>
where
    T: InsertUserQuery,
{
    pub fn new(users: T, auth_keys: CreateAuthKeyService) -> Self {
        Self { users, auth_keys }
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
    async fn call(
        &self,
        req: &'a CreateUser,
    ) -> Result<Self::Response, Self::Error> {
        let user = self.users.call(req).await?;

        let auth_key_params = CreateAuthKey { user_id: user.id };

        let _ = self
            .auth_keys
            .call(&auth_key_params)
            .await?;

        self.auth_keys
            .call(&auth_key_params)
            .await?;

        Ok(user)
    }
}
