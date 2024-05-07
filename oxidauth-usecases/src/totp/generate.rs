use async_trait::async_trait;

use oxidauth_kernel::{error::BoxedError, roles::create_role::*};
use oxidauth_repository::roles::insert_role::InsertRoleQuery;

pub struct GenerateTOTPUseCase<T>
where
    T: SelectUserByIdQuery,
{
    user: T,
}

impl<T> GenerateTOTPUseCase<T>
where
    T: SelectUserByIdQuery,
{
    pub fn new(user: T) -> Self {
        Self { user }
    }
}

#[async_trait]
impl<'a, T> Service<&'a GenerateTOTP> for GenerateTOTPUseCase<T>
where
    T: SelectUserByIdQuery,
{
    type Response = Role;
    type Error = BoxedError;

    #[tracing::instrument(name = "generate_totp_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a GenerateTOTP,
    ) -> Result<Self::Response, Self::Error> {

        // get the secret key for the user by id
        let key_ascii = "12345678901234567890".to_owned();
        let key = 

        // use totp library boring auth to generate

        let mut totp = boringauth::oath::TOTPBuilder::new()
            .ascii_key(&key_ascii)
            .period(42)
            .finalize();

        self.roles.call(req).await
    }
}
