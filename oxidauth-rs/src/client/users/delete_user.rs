use oxidauth_http::{
    response::Response,
    server::api::v1::users::delete_user_by_id::DeleteUserByIdRes,
};
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "delete_user";

impl Client {
    pub async fn delete_user<T>(
        &self,
        user_id: T,
    ) -> Result<DeleteUserByIdRes, BoxedError>
    where
        T: Into<Uuid>,
    {
        let user_id = user_id.into();

        let resp: Response<DeleteUserByIdRes> = self
            .delete("/users/{}", user_id)
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}
