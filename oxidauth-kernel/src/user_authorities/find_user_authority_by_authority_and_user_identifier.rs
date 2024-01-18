use crate::dev_prelude::*;

pub use super::UserAuthority;

#[async_trait]
pub trait FindUserAuthorityByAuthorityIdAndUserIdentifierService:
    Send + Sync + 'static
{
    async fn find_user_authority_by_authority_id_and_user_identifier(
        &self,
        params: &FindUserAuthorityByAuthorityIdAndUserIdentifierParams,
    ) -> Result<
        UserAuthority,
        FindUserAuthorityByAuthorityIdAndUserIdentifierError,
    >;
}

#[derive(Debug)]
pub struct FindUserAuthorityByAuthorityIdAndUserIdentifierParams {
    pub user_identifier: String,
    pub authority_id: Uuid,
}

#[derive(Debug)]
pub struct FindUserAuthorityByAuthorityIdAndUserIdentifierError {}
