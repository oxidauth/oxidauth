use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::find_users_by_ids::{
    FindUsersByIdsReq, FindUsersByIdsRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "find_users_by_ids";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn find_users_by_ids<T>(
        &self,
        params: T,
    ) -> Result<FindUsersByIdsRes, BoxedError>
    where
        T: Into<FindUsersByIdsReq> + fmt::Debug,
    {
        let params = params.into();

        let resp: Response<FindUsersByIdsRes> = self
            .post("/users", Some(params))
            .await?;

        let users_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(users_res)
    }
}
