use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    user_authorities::delete_user_authority::{
        DeleteUserAuthority, DeleteUserAuthorityTrait, UserAuthority,
    },
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
impl<T> DeleteUserAuthorityTrait for DeleteUserAuthorityUseCase<T>
where
    T: DeleteUserAuthorityQuery,
{
    #[tracing::instrument(name = "delete_user_authority_usecase", skip(self))]
    async fn delete_user_authority(
        &self,
        params: &DeleteUserAuthority,
    ) -> Result<UserAuthority, BoxedError> {
        self.user_authorities
            .call(params)
            .await
    }
}
