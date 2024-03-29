use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::list_all_users::{
    ListAllUsersReq, ListAllUsersRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "list_all_users";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn list_all_users<T>(
        &self,
        params: T,
    ) -> Result<ListAllUsersRes, BoxedError>
    where
        T: Into<ListAllUsersReq> + fmt::Debug,
    {
        let params = params.into();

        let resp: Response<ListAllUsersRes> = self
            .get("/users", params)
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}
