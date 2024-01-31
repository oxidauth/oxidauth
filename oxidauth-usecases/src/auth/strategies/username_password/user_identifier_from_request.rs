use async_trait::async_trait;
use oxidauth_kernel::{auth::UserIdentifierFromRequest, error::BoxedError};

use super::{authenticator::AuthenticateParams, UsernamePassword};

#[async_trait]
impl UserIdentifierFromRequest for UsernamePassword {
    #[tracing::instrument(
        name = "user_identifier from username_password",
        skip(self)
    )]
    async fn user_identifier_from_request(
        &self,
        params: &serde_json::Value,
    ) -> Result<String, BoxedError> {
        let AuthenticateParams { username, .. } =
            serde_json::from_value(params.clone())?;

        Ok(username)
    }
}
