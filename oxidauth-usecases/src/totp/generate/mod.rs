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

        // EMAIL -------------------------------------------------------

        // Get user email
        let user = self
            .users
            .call(&FindUserById {
                user_id: req.user_id,
            })
            .await?;

        // Template
        let template = include_str!("./totp_code.tmpl");

        // Content replacements
        let code: String = totp.generate();

        let text = template.replace("{{code}}", &code);

        let to = match (user.first_name, user.email) {
            (None, Some(email)) => email,
            (Some(first_name), Some(email)) => {
                format!("{} <{}>", first_name, email)
            },
            _ => return Err("missing email".into()),
        };

        let message = Message::builder()
            .from(&self.oxidauth_from)
            .to(&to)
            .subject("MFA Code from Mindly") // TODO(berkeleycole) - what is the best way to get the name mindly dynamically?
            .text(&text)
            .build()
            .map_err(|err| err.to_string())?;

        dbg!(&text);
        dbg!(&message);

        self.sender_service
            .send(&message)
            .await
            .map_err(|err| err.to_string())?;

        Ok(TOTPGenerationRes { success: true })
    }
}
