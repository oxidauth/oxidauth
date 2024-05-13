use std::env::var;
use std::error::Error;
use std::fmt;

use lettre::address::AddressError;
use lettre::{
    transport::smtp::{authentication::Credentials, response::Response},
    SmtpTransport, Transport,
};
use mindly_kernel::mailer::service::{IntoSenderService, SenderService};
use mindly_kernel::mailer::{Message, SendError, SendErrorKind, Sender};

const SMTP_MAILER_HOSTNAME: &str = "SMTP_MAILER_HOSTNAME";
const SMTP_MAILER_USERNAME: &str = "SMTP_MAILER_USERNAME";
const SMTP_MAILER_PASSWORD: &str = "SMTP_MAILER_PASSWORD";

#[derive(Debug, Clone)]
pub struct Smtp {
    host: String,
    username: String,
    password: String,
}

impl Smtp {
    pub fn new(host: String, username: String, password: String) -> Self {
        Self {
            host,
            username,
            password,
        }
    }

    pub fn from_env() -> Result<Self, String> {
        let host = var(SMTP_MAILER_HOSTNAME).map_err(|err| err.to_string())?;

        let username =
            var(SMTP_MAILER_USERNAME).map_err(|err| err.to_string())?;

        let password =
            var(SMTP_MAILER_PASSWORD).map_err(|err| err.to_string())?;

        Ok(Self::new(
            host,
            username,
            password,
        ))
    }
}

impl Sender for Smtp {
    type Value = Response;

    async fn send(&self, msg: &Message) -> Result<Self::Value, SendError> {
        let credentials = Credentials::new(
            self.username.clone(),
            self.password.clone(),
        );

        let mailer = SmtpTransport::relay(&self.host)
            .map_err(|err| SmtpError {
                reason: format!("bad relay: {err:?}"),
            })?
            .credentials(credentials)
            .build();

        let msg: SmtpMessage = msg.into();
        let msg: lettre::Message = msg
            .try_into()
            .map_err(|err| SmtpError {
                reason: format!("failed to send: {err:?}"),
            })?;

        let result = mailer
            .send(&msg)
            .map_err(|err| SmtpError {
                reason: format!("failed to send: {err:?}"),
            })?;

        Ok(result)
    }
}

#[derive(Debug)]
pub struct SmtpError {
    reason: String,
}

impl fmt::Display for SmtpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SmtpError: {}",
            self.reason
        )
    }
}

impl From<SmtpError> for SendError {
    fn from(source: SmtpError) -> Self {
        SendError {
            kind: SendErrorKind::SendFailed,
            source: Some(Box::new(source)),
        }
    }
}

pub struct SmtpMessage<'a>(&'a Message);

impl<'a> From<&'a Message> for SmtpMessage<'a> {
    fn from(msg: &'a Message) -> Self {
        SmtpMessage(msg)
    }
}

impl<'a> TryFrom<SmtpMessage<'a>> for lettre::Message {
    type Error = String;

    fn try_from(msg: SmtpMessage) -> Result<Self, Self::Error> {
        let from = msg
            .0
            .from
            .parse()
            .map_err(|err: AddressError| err.to_string())?;

        let to = msg
            .0
            .to
            .parse()
            .map_err(|err: AddressError| err.to_string())?;

        let msg = Self::builder()
            .from(from)
            .to(to)
            .subject(&msg.0.subject)
            .body(msg.0.text.clone())
            .map_err(|err| err.to_string())?;

        Ok(msg)
    }
}

impl Error for SmtpError {}

impl IntoSenderService<Smtp> for Smtp {
    fn into_sender_service(self) -> SenderService<Smtp> {
        SenderService::new(self)
    }
}
