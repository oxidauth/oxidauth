use async_trait::async_trait;

use oxidauth_kernel::{
    authorities::delete_authority::{Authority, DeleteAuthority, DeleteAuthorityTrait},
    error::BoxedError,
};
use oxidauth_repository::authorities::delete_authority::DeleteAuthorityQuery;

pub struct DeleteAuthorityUseCase<T>
where
    T: DeleteAuthorityQuery,
{
    authorities: T,
}

impl<T> DeleteAuthorityUseCase<T>
where
    T: DeleteAuthorityQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<T> DeleteAuthorityTrait for DeleteAuthorityUseCase<T>
where
    T: DeleteAuthorityQuery,
{
    #[tracing::instrument(name = "delete_authority_usecase", skip(self))]
    async fn delete_authority(
        &self,
        req: &DeleteAuthority,
    ) -> Result<Authority, BoxedError> {
        self.authorities
            .call(req)
            .await
    }
}
