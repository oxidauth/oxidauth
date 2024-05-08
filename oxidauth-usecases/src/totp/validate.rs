use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    service::Service,
    totp::{
        validate::{ValidateTOTP, ValidateTOTPReq},
        TOTPValidation,
    },
};
use oxidauth_repository::totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery;

pub struct ValidateTOTPUseCase<T>
where
    T: SelectTOTPSecrețByUserIdQuery,
{
    secret: T,
}

impl<T> ValidateTOTPUseCase<T>
where
    T: SelectTOTPSecrețByUserIdQuery,
{
    pub fn new(secret: T) -> Self {
        Self { secret }
    }
}

#[async_trait]
impl<'a, T> Service<&'a ValidateTOTP> for ValidateTOTPUseCase<T>
where
    T: SelectTOTPSecrețByUserIdQuery,
{
    type Response = TOTPValidation;
    type Error = BoxedError;

    #[tracing::instrument(name = "validate_totp_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a ValidateTOTP,
    ) -> Result<Self::Response, Self::Error> {
        // prepare TOTP secret params
        let secret_params = SelectTOTPSecretByUserId {
            user_id: req.user_id,
        };

        // get the secret key for the user by id
        let secret_by_user_id: TOTPSecret = self
            .secret
            .call(&secret_params)
            .await?;

        // generate the totp code using secret, 5 min period
        let totp = boringauth::oath::TOTPBuilder::new()
            .ascii_key(
                &secret_by_user_id
                    .secret
                    .to_string(),
            )
            .period(300)
            .finalize();

        // check if newly generated code matches user provided code
        let result = match totp {
            valid if totp == params.code => true,
            _ => false,
        };

        // return true if the code matched, false if not
        let response = TOTPValidation {
            code_validation: result,
        };
    }
}
