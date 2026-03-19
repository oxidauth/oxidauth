use async_trait::async_trait;
use uuid::Uuid;

use oxidauth_kernel::{
    authorities::create_authority::{
        Authority, AuthorityStatus, CreateAuthority, CreateAuthorityTrait,
    },
    error::BoxedError,
};
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
impl<T> CreateAuthorityTrait for CreateAuthorityUseCase<T>
where
    T: InsertAuthorityQuery,
{
    #[tracing::instrument(name = "create_authority_usecase", skip(self))]
    async fn create_authority(
        &self,
        req: &mut CreateAuthority,
    ) -> Result<Authority, BoxedError> {
        if req.client_key.is_none() {
            req.client_key
                .replace(Uuid::new_v4());
        }

        if req.status.is_none() {
            req.status
                .replace(AuthorityStatus::Enabled);
        }

        self.authorities
            .call(req)
            .await
    }
}
