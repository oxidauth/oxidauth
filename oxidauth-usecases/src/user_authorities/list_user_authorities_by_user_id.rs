use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    user_authorities::list_user_authorities_by_user_id::{
        ListUserAuthoritiesByUserId, ListUserAuthoritiesByUserIdTrait,
        UserAuthorityWithAuthority,
    },
};
use oxidauth_repository::user_authorities::select_user_authorities_by_user_id::SelectUserAuthoritiesByUserIdQuery;

pub struct ListUserAuthoritiesByUserIdUseCase<T>
where
    T: SelectUserAuthoritiesByUserIdQuery,
{
    user_authorities: T,
}

impl<T> ListUserAuthoritiesByUserIdUseCase<T>
where
    T: SelectUserAuthoritiesByUserIdQuery,
{
    pub fn new(user_authorities: T) -> Self {
        Self { user_authorities }
    }
}

#[async_trait]
impl<T> ListUserAuthoritiesByUserIdTrait
    for ListUserAuthoritiesByUserIdUseCase<T>
where
    T: SelectUserAuthoritiesByUserIdQuery,
{
    #[tracing::instrument(
        name = "list_user_authorities_by_user_id_usecase",
        skip(self)
    )]
    async fn list_user_authorities_by_user_id(
        &self,
        params: &ListUserAuthoritiesByUserId,
    ) -> Result<Vec<UserAuthorityWithAuthority>, BoxedError> {
        self.user_authorities
            .call(params)
            .await
    }
}
