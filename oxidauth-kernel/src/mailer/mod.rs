pub mod message;
pub mod service;

use std::error::Error;
use std::fmt;

pub use message::*;

pub trait Sender: Send + Sync + 'static {
    type Value;

    fn send(
        &self,
        msg: &Message,
    ) -> impl std::future::Future<Output = Result<Self::Value, SendError>> + Send;
}

#[derive(Debug)]
pub struct SendError {
    pub kind: SendErrorKind,
    pub source: Option<Box<(dyn Error + Send + Sync + 'static)>>,
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            SendErrorKind::SendFailed => write!(
                f,
                "failed to send message: {:?}",
                self.source
            ),
        }
    }
}

impl Error for SendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.source {
            Some(source) => Some(source.as_ref()),
            None => None,
        }
    }
}

#[derive(Debug)]
pub enum SendErrorKind {
    SendFailed,
}
