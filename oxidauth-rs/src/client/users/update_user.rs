use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::update_user::{
    UpdateUserBodyReq, UpdateUserRes,
};
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "update_user";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn update_user<T, U>(
        &self,
        user_id: T,
        user: U,
    ) -> Result<UpdateUserRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug,
        U: Into<UpdateUserBodyReq> + fmt::Debug,
    {
        let user_id = user_id.into();
        let user = user.into();

        let resp: Response<UpdateUserRes> = self
            .post(
                &format!("/users/{}", user_id),
                user,
            )
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}
