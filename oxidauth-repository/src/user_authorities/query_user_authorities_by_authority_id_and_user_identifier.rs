use oxidauth_kernel::user_authorities::find_user_authority_by_authority_and_user_identifier::FindUserAuthorityByAuthorityIdAndUserIdentifierParams;

use crate::prelude::*;

use super::UserAuthorityRow;

#[async_trait]
pub trait QueryUserAuthoritiesByAuthorityIdAndUserIdentifier:
    Send + Sync + 'static
{
    async fn query_user_authorities_by_authority_id_and_user_identifier(
        &self,
        params: &QueryUserAuthoritiesByAuthorityIdAndUserIdentifierParams,
    ) -> Result<
        UserAuthorityRow,
        QueryUserAuthoritiesByAuthorityIdAndUserIdentifierError,
    >;
}

pub type QueryUserAuthoritiesByAuthorityIdAndUserIdentifierParams =
    FindUserAuthorityByAuthorityIdAndUserIdentifierParams;

#[derive(Debug)]
pub struct QueryUserAuthoritiesByAuthorityIdAndUserIdentifierError {}
