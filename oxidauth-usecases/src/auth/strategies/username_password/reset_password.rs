use async_trait::async_trait;

use oxidauth_kernel::{
    auth::username_password::reset_password::ResetPasswordInfo, authorities::create_authority::*,
    error::BoxedError,
};
use oxidauth_repository::authorities::insert_authority::InsertAuthorityQuery;

pub struct ResetPasswordUseCase<U>
where
    V: ValidateTotp,
{
    users: U,
}

impl<U> ResetPasswordUseCase<U>
where
    U: SelectUserByEmail,
{
    pub fn new(users: U) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<'a, U> Service<&'a mut ResetPasswordInfo> for ResetPasswordUseCase<U>
where
    U: InsertAuthorityQuery,
{
    type Response = Authority;
    type Error = BoxedError;

    #[tracing::instrument(name = "Reset_password_usecase", skip(self))]
    async fn call(&self, req: &'a mut ResetPasswordInfo) -> Result<Self::Response, Self::Error> {
        // TODO: Validate code

        // TODO: Validate password & conf match

        // TODO: Salt & Pepper new password

        // TODO: Set new password on user
    }
}
