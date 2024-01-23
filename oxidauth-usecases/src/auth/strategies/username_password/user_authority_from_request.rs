use async_trait::async_trait;
use oxidauth_kernel::{
    auth::UserAuthorityFromRequest, error::BoxedError,
    user_authorities::user_authority_create::UserAuthorityCreate,
};
use serde_json::Value;

use super::UsernamePassword;

#[async_trait]
impl UserAuthorityFromRequest for UsernamePassword {
    async fn user_authority_from_request(
        &self,
        params: Value,
    ) -> Result<UserAuthorityCreate, BoxedError> {
        todo!()
    }
}
