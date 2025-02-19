pub use oxidauth_http::{
    response::Response,
    server::api::v1::auth::register::{
        AuthorityStrategy, RegisterReq, RegisterRes,
    },
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Auth;
const METHOD: &str = "register";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn register<T>(
        &self,
        params: T,
    ) -> Result<RegisterRes, BoxedError>
    where
        T: Into<RegisterReq> + fmt::Debug,
    {
        let params = params.into();

        let resp: Response<RegisterRes> = self
            .post("/auth/register", params)
            .await?;

        let role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_res)
    }
}
