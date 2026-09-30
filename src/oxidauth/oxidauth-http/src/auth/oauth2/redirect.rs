use oxidauth_kernel::auth::oauth2::redirect::Oauth2RedirectParams;
use serde::{Deserialize, Serialize};
use url::Url;

pub type Oauth2RedirectReq = Oauth2RedirectParams;

#[derive(Debug, Serialize, Deserialize)]
pub struct Oauth2RedirectRes {
    pub redirect_url: Url,
}
