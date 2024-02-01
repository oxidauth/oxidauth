use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::authorities::update_user_authority::UpdateUserAuthorityRes;
use oxidauth_kernel::{error::BoxedError, user_authorities::update_user_authority::UpdateUserAuthority};

use super::*;

const RESOURCE: Resource = Resource::UserAuthority;
const METHOD: &str = "update_user_authority";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn update_user_authority<T>(
        &self,
        params: T,
    ) -> Result<UpdateUserAuthorityRes, BoxedError>
    where
        T: Into<UpdateUserAuthority> + fmt::Debug,
    {
        let params = params.into();

        let resp: Response<UpdateUserAuthorityRes> = self
            .put(
                &format!(
                    "/users/{}/authorities/{}",
                    params.user_id, params.authority_id
                ),
                params,
            )
            .await?;

        let user_authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_authority_res)
    }
}
