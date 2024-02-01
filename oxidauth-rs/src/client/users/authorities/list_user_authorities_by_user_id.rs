use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::authorities::list_user_authorities_by_user_id::{
    ListUserAuthoritiesByUserIdReq, ListUserAuthoritiesByUserIdRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserAuthority;
const METHOD: &str = "list_user_authorities_by_user_id";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn list_user_authorities_by_user_id<T>(
        &self,
        params: T,
    ) -> Result<ListUserAuthoritiesByUserIdRes, BoxedError>
    where
        T: Into<ListUserAuthoritiesByUserIdReq> + fmt::Debug,
    {
        let params = params.into();

        let resp: Response<ListUserAuthoritiesByUserIdRes> = self
            .get(
                &format!(
                    "/users/{}/authorities",
                    params.user_id
                ),
                None::<ListUserAuthoritiesByUserIdReq>,
            )
            .await?;

        let user_authorities_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_authorities_res)
    }
}
