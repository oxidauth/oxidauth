use std::fmt::Debug;

use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    service::Service,
    totp::generate::*,
    totp_secrets::{
        select_totp_secret_by_user_id::{
            SelectTOTPSecretByUserId, SelectTOTPSecretByUserIdError,
        },
        TOTPSecret,
    },
};
use oxidauth_repository::totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery;

pub struct GenerateTOTPUseCase<T>
where
    T: SelectTOTPSecrețByUserIdQuery,
{
    secret: T,
}

impl<T> GenerateTOTPUseCase<T>
where
    T: SelectTOTPSecrețByUserIdQuery,
{
    pub fn new(secret: T) -> Self {
        Self { secret }
    }
}

#[async_trait]
impl<'a, T> Service<&'a GenerateTOTP> for GenerateTOTPUseCase<T>
where
    T: SelectTOTPSecrețByUserIdQuery,
{
    type Response = TOTPCode;
    type Error = BoxedError;

    #[tracing::instrument(name = "generate_totp_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a GenerateTOTP,
    ) -> Result<Self::Response, Self::Error> {
        // get the secret key for the user by id
        let secret_params = SelectTOTPSecretByUserId {
            user_id: req.user_id,
        };

        let secret_by_user_id: TOTPSecret = self
            .secret
            .call(&secret_params)
            .await?;

        let totp = boringauth::oath::TOTPBuilder::new()
            .ascii_key(
                &secret_by_user_id
                    .secret
                    .to_string(),
            )
            .period(300)
            .finalize();

        // we need to also send the email

        let code = totp.or(GenerateTOTPError);
        let result = TOTPCode { code };

        // for now, return the code
        result
    }
}
