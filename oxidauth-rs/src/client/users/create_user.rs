use oxidauth_http::{
    response::Response,
    server::api::v1::users::create_user::{CreateUserReq, CreateUserRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "create_user";

impl Client {
    async fn create_user<T>(
        &self,
        user: T,
    ) -> Result<CreateUserRes, BoxedError>
        where
            T: Into<CreateUserReq>,
    {
        let user = user.into();

        let resp: Response<CreateUserRes> = self
            .post("/users", user)
            .await?;

        let user_res = handle_response(
            RESOURCE,
            METHOD,
            resp,
        )?;

        Ok(user_res)
    }
}
