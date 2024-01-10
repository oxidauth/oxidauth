use std::error::Error;
use std::fmt::Debug;

use serde::Serialize;

pub type BoxedError = Box<dyn Error + Send + Sync + 'static>;

#[derive(Debug, Serialize)]
pub struct OxidAuthError<C, E>
where
    C: Debug + Serialize,
    E: Debug + Serialize,
{
    pub name: String,
    pub display: String,
    pub debug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<C>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<E>,
}

pub trait IntoOxidAuthError<C, E>
where
    C: Debug + Serialize,
    E: Debug + Serialize,
{
    fn into_error(&self) -> OxidAuthError<C, E>;
}

impl IntoOxidAuthError<(), String> for Box<dyn Error + Send + Sync + 'static> {
    fn into_error(&self) -> OxidAuthError<(), String> {
        OxidAuthError {
            name: "BoxedError".to_owned(),
            display: format!("{}", self),
            debug: format!("{:?}", self),
            status_code: None,
            context: None,
            source: self
                .source()
                .map(|s| s.to_string()),
        }
    }
}
