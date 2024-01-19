use async_trait::async_trait;
use uuid::Uuid;

use oxidauth_kernel::{authorities::create_authority::*, error::BoxedError};
use oxidauth_repository::authorities::insert_authority::InsertAuthorityQuery;

pub struct CreateAuthorityUseCase<T>
where
    T: InsertAuthorityQuery,
{
    authorities: T,
}

impl<T> CreateAuthorityUseCase<T>
where
    T: InsertAuthorityQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<'a, T> Service<&'a mut CreateAuthority> for CreateAuthorityUseCase<T>
where
    T: InsertAuthorityQuery,
{
    type Response = Authority;
    type Error = BoxedError;

    #[tracing::instrument(name = "create_authority_usecase", skip(self))]
    async fn call(&self, req: &'a mut CreateAuthority) -> Result<Self::Response, Self::Error> {
        if req.client_key.is_none() {
            req.client_key.replace(Uuid::new_v4());
        }

        if req.status.is_none() {
            req.status.replace(AuthorityStatus::Enabled);
        }

        self.authorities
            .call(req)
            .await
    }
}

