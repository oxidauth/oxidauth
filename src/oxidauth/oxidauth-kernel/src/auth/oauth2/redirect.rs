use std::{fmt, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::dev_prelude::BoxedError;

#[async_trait]
pub trait Oauth2RedirectServiceTrait: Send + Sync + 'static {
    async fn oauth2_redirect(
        &self,
        params: &Oauth2RedirectParams,
    ) -> Result<Oauth2RedirectResponse, BoxedError>;
}

pub type Oauth2RedirectService = Arc<dyn Oauth2RedirectServiceTrait>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Oauth2RedirectParams {
    pub client_key: Uuid,
    pub email: Option<String>,
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
            Unknown(value) => write!(f, "unable to create redirect_url: {}", value),
        }
    }
}
