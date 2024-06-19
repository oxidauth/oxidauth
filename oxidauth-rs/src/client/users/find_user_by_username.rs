use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::find_user_by_username::FindUserByUsernameRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "find_user_by_username";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn find_user_by_username<T>(
        &self,
        username: T,
    ) -> Result<FindUserByUsernameRes, BoxedError>
    where
        T: Into<String> + fmt::Debug,
    {
        let username = username.into();

        let resp: Response<FindUserByUsernameRes> = self
            .get(
                &format!(
                    "/users/by_username/{}",
                    username,
                ),
                None::<()>,
            )
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}
