use async_trait::async_trait;
use uuid::Uuid;

use oxidauth_kernel::{authorities::update_authority::*, error::BoxedError};
use oxidauth_repository::authorities::select_authority_by_id::*;
use oxidauth_repository::authorities::update_authority::UpdateAuthorityQuery;

pub struct UpdateAuthorityUseCase<T, I>
where
    T: UpdateAuthorityQuery,
    I: SelectAuthorityByIdQuery,
{
    update_authority: T,
    authority_by_id: I,
}

impl<T, I> UpdateAuthorityUseCase<T, I>
where
    T: UpdateAuthorityQuery,
    I: SelectAuthorityByIdQuery,
{
    pub fn new(update_authority: T, authority_by_id: I) -> Self {
        Self {
            update_authority,
            authority_by_id,
        }
    }
}

#[async_trait]
impl<'a, T, I> Service<&'a mut UpdateAuthority> for UpdateAuthorityUseCase<T, I>
where
    T: UpdateAuthorityQuery,
    I: SelectAuthorityByIdQuery,
{
    type Response = Authority;
    type Error = BoxedError;

    #[tracing::instrument(name = "update_authority_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a mut UpdateAuthority,
    ) -> Result<Self::Response, Self::Error> {
        // NOTE(kazenix): the id is getting set in the http handler from the url param, this is
        // just for anything else that might use this service.
        let authority_id = req
            .id
            .ok_or("UpdateAuthority must have authority_id")?;

        let current = self
            .authority_by_id
            .call(&FindAuthorityById { authority_id })
            .await?;

        if req.client_key.is_none() {
            req.client_key
                .replace(Uuid::new_v4());
        }

        if req.status.is_none() {
            req.status
                .replace(current.status);
        }

        self.update_authority
            .call(req)
            .await
    }
}
