use async_trait::async_trait;

use oxidauth_kernel::{authorities::find_authority_by_strategy::*, error::BoxedError};
use oxidauth_repository::authorities::select_authority_by_strategy::SelectAuthorityByStrategyQuery;

pub struct FindAuthorityByStrategyUseCase<T>
where
    T: SelectAuthorityByStrategyQuery,
{
    authorities: T,
}

impl<T> FindAuthorityByStrategyUseCase<T>
where
    T: SelectAuthorityByStrategyQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<'a, T> Service<&'a FindAuthorityByStrategy> for FindAuthorityByStrategyUseCase<T>
where
    T: SelectAuthorityByStrategyQuery,
{
    type Response = Authority;
    type Error = BoxedError;

    #[tracing::instrument(name = "find_authority_by_strategy_usecase", skip(self))]
    async fn call(&self, req: &'a FindAuthorityByStrategy) -> Result<Self::Response, Self::Error> {
        self.authorities
            .call(req)
            .await
    }
}
