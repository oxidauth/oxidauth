use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, user_authorities::UserAuthority};
use uuid::Uuid;

#[async_trait]
pub trait SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery: Send + Sync + 'static {
    async fn select_user_authority_by_authority_id_and_user_identifier(
        &self,
        params: &SelectUserAuthorityByAuthorityIdAndUserIdentifierQueryParams,
    ) -> Result<UserAuthority, BoxedError>;
}
#[derive(Debug)]
pub struct SelectUserAuthorityByAuthorityIdAndUserIdentifierQueryParams {
    pub authority_id: Uuid,
    pub user_identifier: String,
}
