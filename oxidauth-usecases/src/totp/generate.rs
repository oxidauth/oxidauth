use async_trait::async_trait;

use crate::mailer::smtp::Smtp;
use boringauth::oath::TOTPBuilder;
use oxidauth_kernel::{
    error::BoxedError,
    mailer::{service::SenderService, Message},
    service::Service,
    totp::generate::*,
    totp_secrets::{
        find_totp_secret_by_user_id::FindTOTPSecretByUserId, TOTPSecret,
    },
    users::find_user_by_id::FindUserById,
};
use oxidauth_repository::{
    totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery,
    users::select_user_by_id_query::SelectUserByIdQuery,
};

#[derive(Clone)]
pub struct GenerateTOTPUseCase<T, U>
where
    T: SelectTOTPSecrețByUserIdQuery,
    U: SelectUserByIdQuery,
{
    secret: T,
    users: U,
    sender_service: SenderService<Smtp>,
    oxidauth_from: String,
}

impl<T, U> GenerateTOTPUseCase<T, U>
where
    T: SelectTOTPSecrețByUserIdQuery,
    U: SelectUserByIdQuery,
{
    pub fn new(
        secret: T,
        users: U,
        sender_service: &SenderService<Smtp>,
        oxidauth_from: &str,
    ) -> Self {
        Self {
            secret,
            users,
            sender_service: sender_service.clone(),
            oxidauth_from: oxidauth_from.to_string(),
        }
    }
}

#[async_trait]
impl<'a, T, U> Service<&'a GenerateTOTP> for GenerateTOTPUseCase<T, U>
where
    T: SelectTOTPSecrețByUserIdQuery,
    U: SelectUserByIdQuery,
{
    type Response = TOTPGenerationRes;
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
        let totp = TOTPBuilder::new()
            .ascii_key(&secret_by_user_id.secret)
            .period(300)
            .finalize()
            .expect("Could not generate totp"); // this is probably a cop out

        Ok(TOTPGenerationRes { success: true })
    }
}
