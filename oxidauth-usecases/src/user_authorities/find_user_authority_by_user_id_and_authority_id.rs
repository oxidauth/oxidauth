use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    user_authorities::find_user_authority_by_user_id_and_authority_id::{
        FindUserAuthorityByUserIdAndAuthorityId,
        FindUserAuthorityByUserIdAndAuthorityIdTrait,
        UserAuthorityWithAuthority,
    },
};
use oxidauth_repository::user_authorities::select_user_authority_by_user_id_and_authority_id::SelectUserAuthorityByUserIdAndAuthorityIdQuery;

pub struct FindUserAuthorityByUserIdAndAuthorityIdUseCase<T>
where
    T: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
{
    user_authorities: T,
}

impl<T> FindUserAuthorityByUserIdAndAuthorityIdUseCase<T>
where
    T: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
{
    pub fn new(user_authorities: T) -> Self {
        Self { user_authorities }
    }
}

#[async_trait]
impl<T> FindUserAuthorityByUserIdAndAuthorityIdTrait
    for FindUserAuthorityByUserIdAndAuthorityIdUseCase<T>
where
    T: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
{
    #[tracing::instrument(
        name = "find_user_authority_by_user_id_and_authority_id_usecase",
        skip(self)
    )]
    async fn find_user_authority_by_user_id_and_authority_id(
        &self,
        params: &FindUserAuthorityByUserIdAndAuthorityId,
    ) -> Result<UserAuthorityWithAuthority, BoxedError> {
        self.user_authorities
            .call(params)
            .await
    }
}
