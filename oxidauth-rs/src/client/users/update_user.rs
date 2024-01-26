use uuid::Uuid;
use oxidauth_http::{
    response::Response,
    server::api::v1::users::update_user::{UpdateUserBodyReq, UpdateUserRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "update_user";

impl Client {
    async fn update_user<T, U>(
        &self,
        user_id: T,
        user: U,
    ) -> Result<UpdateUserRes, BoxedError>
        where
            T: Into<Uuid>,
            U: Into<UpdateUserBodyReq>,
    {
        let user_id = user_id.into();
        let user = user.into();

        let resp: Response<UpdateUserRes> = self
            .post("/users/{}", user)
            .await?;

        let user_res = handle_response(
            RESOURCE,
            METHOD,
            resp,
        )?;

        Ok(user_res)
    }
}
