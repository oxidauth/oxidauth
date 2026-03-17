use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::find_user_by_id::{
    FindUserByIdReq, FindUserByIdRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "find_user_by_id";

impl Client {
    #[cfg(all(not(feature = "mock")))]
    #[tracing::instrument(skip(self))]
    pub async fn find_user_by_id<T>(
        &self,
        params: T,
    ) -> Result<FindUserByIdRes, BoxedError>
    where
        T: Into<FindUserByIdReq> + fmt::Debug,
    {
        let params = params.into();

        let resp: Response<FindUserByIdRes> = self
            .get(
                &format!("/users/{}", params.user_id),
                None::<()>,
            )
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }

    #[cfg(all(feature = "mock"))]
    #[tracing::instrument(skip(self))]
    pub async fn find_user_by_id<T>(
        &self,
        params: T,
    ) -> Result<FindUserByIdRes, BoxedError>
    where
        T: Into<FindUserByIdReq> + fmt::Debug,
    {
        use chrono::Utc;
        use uuid::Uuid;

        let now = Utc::now();

        Ok(FindUserByIdRes {
            user: User {
                id: Uuid::new_v4(),
                kind: UserKind::Human,
                status: UserStatus::Enabled,
                username: "Test User".to_string(),
                email: Some("test@user.com".to_string()),
                first_name: Some("Test".to_string()),
                last_name: Some("User".to_string()),
                profile: serde_json::json!({}),
                created_at: now,
                updated_at: now
            }})
    }
}
