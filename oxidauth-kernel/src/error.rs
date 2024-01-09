use std::error::Error;
use std::fmt::Debug;

pub type BoxedError = Box<dyn Error + Send + Sync + 'static>;

#[derive(Debug)]
pub struct OxidAuthError<C, E>
where
    C: Debug,
    E: Debug,
{
    pub name: String,
    pub display: String,
    pub debug: String,
    pub status_code: Option<usize>,
    pub context: Option<C>,
    pub source: Option<E>,
}

pub trait IntoOxidAuthError<C, E>
where
    C: Debug,
    E: Debug,
{
    fn into_error(&self) -> OxidAuthError<C, E>;
}

impl IntoOxidAuthError<(), String> for Box<dyn Error> {
    fn into_error(&self) -> OxidAuthError<(), String> {
        OxidAuthError {
            name: "BoxedError".to_owned(),
            display: format!("{}", self),
            debug: format!("{:?}", self),
            status_code: None,
            context: None,
            source: self.source().map(|s| s.to_string()),
        }
    }
}
