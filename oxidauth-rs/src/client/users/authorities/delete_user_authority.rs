use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::authorities::delete_user_authority::{
    DeleteUserAuthorityReq, DeleteUserAuthorityRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserAuthority;
const METHOD: &str = "delete_user_authority";

impl Client {
    pub async fn delete_user_authority<T>(
        &self,
        params: T,
    ) -> Result<DeleteUserAuthorityRes, BoxedError>
    where
        T: Into<DeleteUserAuthorityReq>,
    {
        let params = params.into();

        let resp: Response<DeleteUserAuthorityRes> = self
            .delete(
                &format!(
                    "/users/{}/authorities/{}",
                    params.user_id, params.authority_id
                ),
                None::<DeleteUserAuthorityReq>,
            )
            .await?;

        let user_authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_authority_res)
    }
}
