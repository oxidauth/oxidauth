use async_trait::async_trait;
use oxidauth_kernel::authorities::TotpSettings;
use oxidauth_kernel::totp_secrets::create_totp_secrets_by_authority_id::{
    CreateTotpSecrets, CreateTotpSecretsService,
};
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
    totp_secrets: CreateTotpSecretsService,
}

impl<T, I> UpdateAuthorityUseCase<T, I>
where
    T: UpdateAuthorityQuery,
    I: SelectAuthorityByIdQuery,
{
    pub fn new(
        update_authority: T,
        authority_by_id: I,
        totp_secrets: CreateTotpSecretsService,
    ) -> Self {
        Self {
            update_authority,
            authority_by_id,
            totp_secrets,
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

        if let (TotpSettings::Disabled, TotpSettings::Enabled { .. }) = (
            &current.settings.totp,
            &req.settings.totp,
        ) {
            self.totp_secrets
                .create_totp_secrets(&CreateTotpSecrets { authority_id })
                .await?;
        }

        self.update_authority
            .call(req)
            .await
    }
}
