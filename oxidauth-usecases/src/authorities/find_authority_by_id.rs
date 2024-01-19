use async_trait::async_trait;

use oxidauth_kernel::{authorities::find_authority_by_id::*, error::BoxedError};
use oxidauth_repository::authorities::select_authority_by_id::SelectAuthorityByIdQuery;

pub struct FindAuthorityByIdUseCase<T>
where
    T: SelectAuthorityByIdQuery,
{
    authorities: T,
}

impl<T> FindAuthorityByIdUseCase<T>
where
    T: SelectAuthorityByIdQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<'a, T> Service<&'a FindAuthorityById> for FindAuthorityByIdUseCase<T>
where
    T: SelectAuthorityByIdQuery,
{
    type Response = Authority;
    type Error = BoxedError;

    #[tracing::instrument(name = "find_authority_by_id_usecase", skip(self))]
    async fn call(&self, req: &'a FindAuthorityById) -> Result<Self::Response, Self::Error> {
        self.authorities
            .call(req)
            .await
    }
}
