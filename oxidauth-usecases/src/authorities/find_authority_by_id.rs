use async_trait::async_trait;

use oxidauth_kernel::{
    authorities::find_authority_by_id::{Authority, FindAuthorityById, FindAuthorityByIdTrait},
    error::BoxedError,
};
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
impl<T> FindAuthorityByIdTrait for FindAuthorityByIdUseCase<T>
where
    T: SelectAuthorityByIdQuery,
{
    #[tracing::instrument(name = "find_authority_by_id_usecase", skip(self))]
    async fn find_authority_by_id(
        &self,
        req: &FindAuthorityById,
    ) -> Result<Authority, BoxedError> {
        self.authorities
            .call(req)
            .await
    }
}
