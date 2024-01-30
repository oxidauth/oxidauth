use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::auth::authenticate::{
    AuthenticateReq, AuthenticateRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Auth;
const METHOD: &str = "authenticate";

impl Client {
    pub async fn authenticate<T>(
        &self,
        params: T,
    ) -> Result<AuthenticateRes, BoxedError>
    where
        T: Into<AuthenticateReq>,
    {
        let params = params.into();

        let resp: Response<AuthenticateRes> = self
            .post("/auth/authenticate", params)
            .await?;

        let authenticate_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authenticate_res)
    }
}
