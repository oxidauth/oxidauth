use std::{fmt, sync::Arc};

use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::dev_prelude::{BoxedError, Service};

pub type Oauth2RedirectService = Arc<
    dyn for<'a> Service<
            &'a Oauth2RedirectParams,
            Response = Oauth2RedirectResponse,
            Error = BoxedError,
        >,
>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Oauth2RedirectParams {
    pub client_key: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Oauth2RedirectResponse {
    pub redirect_url: Url,
}

#[derive(Debug)]
pub enum ParseOauth2RedirectUrlError {
    Unknown(String),
}

impl fmt::Display for ParseOauth2RedirectUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseOauth2RedirectUrlError::*;

        match self {
            Unknown(value) => write!(
                f,
                "unable to create redirect_url: {}",
                value
            ),
        }
    }
}
