use oxidauth_http::{
    response::Response,
    server::api::v1::users::list_all_users::{
        ListAllUsersReq, ListAllUsersRes,
    },
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "list_all_users";

impl Client {
    pub async fn list_all_users<T>(
        &self,
        params: T,
    ) -> Result<ListAllUsersRes, BoxedError>
    where
        T: Into<ListAllUsersReq>,
    {
        let params = params.into();

        let resp: Response<ListAllUsersRes> = self
            .get("/users/", params)
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}
