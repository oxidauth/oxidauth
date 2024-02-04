use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::find_user_by_id::{
    FindUserByIdReq, FindUserByIdRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "find_user_by_id";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn find_user_by_id<T>(
        &self,
        params: T,
    ) -> Result<FindUserByIdRes, BoxedError>
    where
        T: Into<FindUserByIdReq> + fmt::Debug,
    {
        let params = params.into();

        let resp: Response<FindUserByIdRes> = self
            .get(
                &format!("/users/{}", params.user_id),
                None::<()>,
            )
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}
