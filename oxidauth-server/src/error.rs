use std::{error::Error as StdError, fmt::Display};

use uuid::Uuid;

#[derive(Debug)]
pub struct StrError(String);

impl Display for StrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl StdError for StrError {}

#[derive(Debug)]
pub enum ErrorSrc {
    Generic(StrError),
    SqlxError(sqlx::Error),
}

#[derive(Debug)]
pub struct Error {
    user: Option<Uuid>,
    resource: String,
    action: String,
    source: Option<ErrorSrc>,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "this is an error")
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        if let Some(source) = &self.source {
            let error: &(dyn StdError + 'static) = match source {
                ErrorSrc::Generic(err) => err,
                ErrorSrc::SqlxError(err) => err,
            };

            return Some(error);
        }

        None
    }
}
