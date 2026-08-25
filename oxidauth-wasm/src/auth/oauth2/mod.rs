pub mod redirect;

use crate::BoxedError;
use crate::response::Response;

use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use super::{Client, fmt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Oauth2RedirectParams {
    pub client_key: Uuid,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Oauth2RedirectRes {
    pub redirect_url: Url,
}
