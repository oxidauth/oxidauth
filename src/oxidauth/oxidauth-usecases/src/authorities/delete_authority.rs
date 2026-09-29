use async_trait::async_trait;

use oxidauth_kernel::{authorities::delete_authority::*, error::BoxedError};
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
impl<'a, T> Service<&'a DeleteAuthority> for DeleteAuthorityUseCase<T>
where
    T: DeleteAuthorityQuery,
{
    type Response = Authority;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_authority_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a DeleteAuthority,
    ) -> Result<Self::Response, Self::Error> {
        self.authorities
            .call(req)
            .await
    }
}
