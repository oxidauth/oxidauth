use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::authorities::create_user_authority::{
    CreateUserAuthorityBodyReq, CreateUserAuthorityRes,
};
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::UserAuthority;
const METHOD: &str = "create_user_authority";

impl Client {
    pub async fn create_user_authority<T, U>(
        &self,
        user_id: T,
        user_authority: U,
    ) -> Result<CreateUserAuthorityRes, BoxedError>
    where
        T: Into<Uuid>,
        U: Into<CreateUserAuthorityBodyReq>,
    {
        let user_id = user_id.into();
        let user_authority = user_authority.into();

        let resp: Response<CreateUserAuthorityRes> = self
            .post(
                &format!(
                    "/users/{}/authorities",
                    user_id
                ),
                user_authority,
            )
            .await?;

        let user_authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_authority_res)
    }
}
