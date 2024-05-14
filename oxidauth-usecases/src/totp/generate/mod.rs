use std::fmt::Debug;

use async_trait::async_trait;

use crate::sender::smtp::Smtp;
use oxidauth_kernel::{
    error::BoxedError,
    mailer::{service::SenderService, Message},
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
    sender_service: SenderService<Smtp>,
    mindly_app_base_url: String,
    mindly_from: String,
}

impl<T> GenerateTOTPUseCase<T>
where
    T: SelectTOTPSecrețByUserIdQuery,
{
    pub fn new(
        secret: T,
        sender_service: &SenderService<Smtp>,
        mindly_app_base_url: &str,
        mindly_from: &str,
    ) -> Self {
        Self {
            secret,
            sender_service: sender_service.clone(),
            mindly_app_base_url: mindly_app_base_url.to_string(),
            mindly_from: mindly_from.to_string(),
        }
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

        let code = totp.or(GenerateTOTPError);
        let result = TOTPCode { code };

        // EMAIL -------------------------------------------------------

        // Template
        let template = include_str!("./totp_code.tmpl");

        // Content replacements
        let text = template.replace("{{code}}", &code);

        let to = format!(
            "{} <{}>",
            &user.name, &user.email
        );

        let message = Message::builder()
            .from(&self.mindly_from)
            .to(&to)
            .subject("Your Mindly Action Plan ")
            .text(&text)
            .build()
            .map_err(|err| err.to_string())?;

        self.sender_service
            .send(&message)
            .await
            .map_err(|err| err.to_string())?;

        // for now, return the code
        Ok(result)
    }
}
