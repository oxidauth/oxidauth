use async_trait::async_trait;

use oxidauth_kernel::totp_secrets::create_totp_secret::{
    CreateTotpSecret, CreateTotpSecretService,
};
use oxidauth_kernel::{error::BoxedError, users::create_user::*};
use oxidauth_repository::users::insert_user::InsertUserQuery;

pub struct CreateUserUseCase<T>
where
    T: InsertUserQuery,
{
    users: T,
    totp_secrets: CreateTotpSecretService,
}

impl<T> CreateUserUseCase<T>
where
    T: InsertUserQuery,
{
    pub fn new(users: T, totp_secrets: CreateTotpSecretService) -> Self {
        Self {
            users,
            totp_secrets,
        }
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

        let totp_secret_params = CreateTotpSecret { user_id: user.id };

        let _ = self
            .totp_secrets
            .call(&totp_secret_params)
            .await?;

        self.totp_secrets
            .call(&totp_secret_params)
            .await?;

        Ok(user)
    }
}
