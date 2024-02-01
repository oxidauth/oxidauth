use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::find_user_by_id::FindUserByIdRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "find_user_by_id";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn find_user_by_id<T>(
        &self,
        user_id: T,
    ) -> Result<FindUserByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug,
    {
        let user_id = user_id.into();

        let resp: Response<FindUserByIdRes> = self
            .get("/users/{}", user_id)
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}
