use oxidauth_http::{
    response::Response,
    server::api::v1::users::find_user_by_username::{FindUserByUsernameReq, FindUserByUsernameRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "find_user_by_username";

impl Client {
    async fn find_user_by_username<T>(
        &self,
        username: T,
    ) -> Result<FindUserByUsernameRes, BoxedError>
        where
            T: Into<String>,
    {
        let username = username.into();

        let resp: Response<FindUserByUsernameRes> = self
            .get("/users/by_username/{}", username)
            .await?;

        let user_res = handle_response(
            RESOURCE,
            METHOD,
            resp,
        )?;

        Ok(user_res)
    }
}
