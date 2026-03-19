use async_trait::async_trait;

use oxidauth_kernel::{
    authorities::{
        find_authority_by_strategy::{
            Authority, FindAuthorityByStrategy, FindAuthorityByStrategyTrait,
        },
        AuthorityNotFoundError,
    },
    error::BoxedError,
};
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
impl<T> FindAuthorityByStrategyTrait for FindAuthorityByStrategyUseCase<T>
where
    T: SelectAuthorityByStrategyQuery,
{
    #[tracing::instrument(
        name = "find_authority_by_strategy_usecase",
        skip(self)
    )]
    async fn find_authority_by_strategy(
        &self,
        params: &FindAuthorityByStrategy,
    ) -> Result<Authority, BoxedError> {
        let authority = self
            .authorities
            .call(params)
            .await?
            .ok_or_else(|| AuthorityNotFoundError::strategy(params.strategy))?;

        Ok(authority)
    }
}
