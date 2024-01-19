use async_trait::async_trait;

use oxidauth_kernel::{authorities::list_all_authorities::*, error::BoxedError};
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
impl<'a, T> Service<&'a ListAllAuthorities> for ListAllAuthoritiesUseCase<T>
where
    T: SelectAllAuthoritiesQuery,
{
    type Response = Vec<Authority>;
    type Error = BoxedError;

    #[tracing::instrument(name = "list_all_authorities_usecase", skip(self))]
    async fn call(&self, req: &'a ListAllAuthorities) -> Result<Self::Response, Self::Error> {
        self.authorities
            .call(req)
            .await
    }
}


