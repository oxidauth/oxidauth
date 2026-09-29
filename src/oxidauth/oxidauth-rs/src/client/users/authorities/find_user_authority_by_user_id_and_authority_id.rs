use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::authorities::find_user_authority_by_user_id_and_authority_id::{
    FindUserAuthorityByUserIdAndAuthorityIdReq, FindUserAuthorityByUserIdAndAuthorityIdRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserAuthority;
const METHOD: &str = "find_user_authority_by_user_id_and_authority_id";

#[async_trait]
pub trait FindUserAuthorityByUserIdAndAuthorityIdTrait {
    async fn find_user_authority_by_user_id_and_authority_id<T>(
        &self,
        params: T,
    ) -> Result<FindUserAuthorityByUserIdAndAuthorityIdRes, BoxedError>
    where
        T: Into<FindUserAuthorityByUserIdAndAuthorityIdReq> + fmt::Debug + Send;
}

#[async_trait]
impl FindUserAuthorityByUserIdAndAuthorityIdTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_user_authority_by_user_id_and_authority_id<T>(
        &self,
        params: T,
    ) -> Result<FindUserAuthorityByUserIdAndAuthorityIdRes, BoxedError>
    where
        T: Into<FindUserAuthorityByUserIdAndAuthorityIdReq> + fmt::Debug + Send,
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

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl FindUserAuthorityByUserIdAndAuthorityIdTrait for ClientMock {
    async fn find_user_authority_by_user_id_and_authority_id<T>(
        &self,
        params: T,
    ) -> Result<FindUserAuthorityByUserIdAndAuthorityIdRes, BoxedError>
    where
        T: Into<FindUserAuthorityByUserIdAndAuthorityIdReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .find_user_authority_by_user_id_and_authority_id_fn
            .clone()
        else {
            panic!(
                "find_user_authority_by_user_id_and_authority_id not defined for mock client"
            );
        };

        return func(params.into());
    }
}
