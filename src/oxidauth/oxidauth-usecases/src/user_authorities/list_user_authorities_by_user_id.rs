use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, service::Service,
    user_authorities::list_user_authorities_by_user_id::*,
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
impl<'a, T> Service<&'a ListUserAuthoritiesByUserId>
    for ListUserAuthoritiesByUserIdUseCase<T>
where
    T: SelectUserAuthoritiesByUserIdQuery,
{
    type Response = Vec<UserAuthorityWithAuthority>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "list_user_authorities_by_user_id_usecase",
        skip(self)
    )]
    async fn call(
        &self,
        req: &'a ListUserAuthoritiesByUserId,
    ) -> Result<Self::Response, Self::Error> {
        self.user_authorities
            .call(req)
            .await
    }
}
