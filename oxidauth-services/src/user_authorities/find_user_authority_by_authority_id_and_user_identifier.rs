use oxidauth_kernel::user_authority::find_user_authority_by_authority_and_user_identifier::*;
use oxidauth_repository::user_authorities::query_user_authorities_by_authority_id_and_user_identifier::QueryUserAuthoritiesByAuthorityIdAndUserIdentifier;

use crate::dev_prelude::*;

pub struct FindUserAuthorityByAuthorityIdAndUserIdentifierUseCase<R>
where
    R: QueryUserAuthoritiesByAuthorityIdAndUserIdentifier,
{
    repo: R,
}

#[async_trait]
impl<R> FindUserAuthorityByAuthorityIdAndUserIdentifierService
    for FindUserAuthorityByAuthorityIdAndUserIdentifierUseCase<R>
where
    R: QueryUserAuthoritiesByAuthorityIdAndUserIdentifier,
{
    async fn find_user_authority_by_authority_id_and_user_identifier(
        &self,
        params: &FindUserAuthorityByAuthorityIdAndUserIdentifierParams,
    ) -> Result<UserAuthority, FindUserAuthorityByAuthorityIdAndUserIdentifierError> {
        let result = self
            .repo
            .query_user_authorities_by_authority_id_and_user_identifier(params)
            .await
            .map_err(|_| FindUserAuthorityByAuthorityIdAndUserIdentifierError {})?;

        Ok(result)
    }
}
