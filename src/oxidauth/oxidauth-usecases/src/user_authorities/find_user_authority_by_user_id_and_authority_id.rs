use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, service::Service, user_authorities::find_user_authority_by_user_id_and_authority_id::*,
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
impl<'a, T> Service<&'a FindUserAuthorityByUserIdAndAuthorityId>
    for FindUserAuthorityByUserIdAndAuthorityIdUseCase<T>
where
    T: SelectUserAuthorityByUserIdAndAuthorityIdQuery,
{
    type Response = UserAuthorityWithAuthority;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "find_user_authority_by_user_id_and_authority_id_usecase",
        skip(self)
    )]
    async fn call(
        &self,
        req: &'a FindUserAuthorityByUserIdAndAuthorityId,
    ) -> Result<Self::Response, Self::Error> {
        self.user_authorities
            .call(req)
            .await
    }
}
