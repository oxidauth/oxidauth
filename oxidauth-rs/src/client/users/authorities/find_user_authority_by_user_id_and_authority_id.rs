use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::authorities::find_user_authority_by_user_id_and_authority_id::{
    FindUserAuthorityByUserIdAndAuthorityIdReq, FindUserAuthorityByUserIdAndAuthorityIdRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserAuthority;
const METHOD: &str = "find_user_authority_by_user_id_and_authority_id";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn find_user_authority_by_user_id_and_authority_id<T>(
        &self,
        params: T,
    ) -> Result<FindUserAuthorityByUserIdAndAuthorityIdRes, BoxedError>
    where
        T: Into<FindUserAuthorityByUserIdAndAuthorityIdReq> + fmt::Debug,
    {
        let params = params.into();

        let resp: Response<FindUserAuthorityByUserIdAndAuthorityIdRes> = self
            .get(
                &format!(
                    "/users/{}/authorities/{}",
                    params.user_id, params.authority_id
                ),
                None::<FindUserAuthorityByUserIdAndAuthorityIdReq>,
            )
            .await?;

        let user_authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_authority_res)
    }
}
