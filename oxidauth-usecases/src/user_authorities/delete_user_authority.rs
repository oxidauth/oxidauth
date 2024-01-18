use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, service::Service,
    user_authorities::delete_user_authority::*,
};
use oxidauth_repository::user_authorities::delete_user_authority::DeleteUserAuthorityQuery;

pub struct DeleteUserAuthorityUseCase<T>
where
    T: DeleteUserAuthorityQuery,
{
    user_authorities: T,
}

impl<T> DeleteUserAuthorityUseCase<T>
where
    T: DeleteUserAuthorityQuery,
{
    pub fn new(user_authorities: T) -> Self {
        Self { user_authorities }
    }
}

#[async_trait]
impl<'a, T> Service<&'a DeleteUserAuthority> for DeleteUserAuthorityUseCase<T>
where
    T: DeleteUserAuthorityQuery,
{
    type Response = UserAuthority;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_user_authority_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a DeleteUserAuthority,
    ) -> Result<Self::Response, Self::Error> {
        self.user_authorities
            .call(req)
            .await
    }
}
