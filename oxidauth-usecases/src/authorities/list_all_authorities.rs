use async_trait::async_trait;

use oxidauth_kernel::{
    authorities::list_all_authorities::{
        Authority, ListAllAuthorities, ListAllAuthoritiesTrait,
    },
    error::BoxedError,
};
use oxidauth_repository::authorities::select_all_authorities::SelectAllAuthoritiesQuery;

pub struct ListAllAuthoritiesUseCase<T>
where
    T: SelectAllAuthoritiesQuery,
{
    authorities: T,
}

impl<T> ListAllAuthoritiesUseCase<T>
where
    T: SelectAllAuthoritiesQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<T> ListAllAuthoritiesTrait for ListAllAuthoritiesUseCase<T>
where
    T: SelectAllAuthoritiesQuery,
{
    #[tracing::instrument(name = "list_all_authorities_usecase", skip(self))]
    async fn list_all_authorities(
        &self,
        req: &ListAllAuthorities,
    ) -> Result<Vec<Authority>, BoxedError> {
        self.authorities
            .call(req)
            .await
    }
}
