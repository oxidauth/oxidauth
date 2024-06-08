use async_trait::async_trait;

use boringauth::oath::TOTPBuilder;
use oxidauth_kernel::{
    error::BoxedError,
    service::Service,
    totp::generate::*,
    totp_secrets::{
        find_totp_secret_by_user_id::FindTOTPSecretByUserId, TOTPSecret,
    },
};
use oxidauth_repository::totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery;

#[derive(Clone)]
pub struct GenerateTOTPUseCase<T>
where
    T: SelectTOTPSecrețByUserIdQuery,
{
    // TODO(dewey4iv): maybe we need to use something else here
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
        let secret_by_user_id: TOTPSecret = self
            .secret
            .call(&FindTOTPSecretByUserId {
                user_id: req.user_id,
            })
            .await?;

        // generate the totp code using secret, 5 min period
        let code = TOTPBuilder::new()
            .ascii_key(&secret_by_user_id.secret)
            .period(300) // TODO(berkeleycole): make this come from authority settings
            .finalize()
            .map_err(|err| {
                format!(
                    "error generating totp: {:?}",
                    err
                )
            })?
            .generate();

        Ok(TOTPCode { code })
    }
}
