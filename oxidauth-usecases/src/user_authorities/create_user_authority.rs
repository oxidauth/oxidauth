use async_trait::async_trait;

use oxidauth_kernel::{
    authorities::{AuthorityNotFoundError, TotpSettings},
    error::BoxedError,
    totp_secrets::create_totp_secret::{
        CreateTotpSecret, CreateTotpSecretService,
    },
    user_authorities::create_user_authority::*,
};
use oxidauth_repository::{
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    user_authorities::insert_user_authority::InsertUserAuthorityQuery,
};

use crate::auth::register::build_registrar;

pub struct CreateUserAuthorityUseCase<A, U>
where
    A: SelectAuthorityByClientKeyQuery,
    U: InsertUserAuthorityQuery,
{
    authority_by_client_key: A,
    insert_user_authority: U,
    totp_secrets: CreateTotpSecretService,
}

impl<A, U> CreateUserAuthorityUseCase<A, U>
where
    A: SelectAuthorityByClientKeyQuery,
    U: InsertUserAuthorityQuery,
{
    pub fn new(
        authority_by_client_key: A,
        insert_user_authority: U,
        totp_secrets: CreateTotpSecretService,
    ) -> Self {
        Self {
            authority_by_client_key,
            insert_user_authority,
            totp_secrets,
        }
    }
}

#[async_trait]
impl<'a, A, U> Service<&'a CreateUserAuthorityParams>
    for CreateUserAuthorityUseCase<A, U>
where
    A: SelectAuthorityByClientKeyQuery,
    U: InsertUserAuthorityQuery,
{
    type Response = UserAuthority;
    type Error = BoxedError;

    #[tracing::instrument(name = "create_user_authority_usecase", skip(self))]
    async fn call(
        &self,
        params: &'a CreateUserAuthorityParams,
    ) -> Result<Self::Response, Self::Error> {
        let authority = self
            .authority_by_client_key
            .call(&params.into())
            .await?
            .ok_or_else(|| {
                AuthorityNotFoundError::client_key(params.client_key)
            })?;

        let registrar = build_registrar(&authority).await?;

        let user_authority = registrar
            .user_authority_from_request(params.params.clone())
            .await?;

        // If user's authority requires 2FA, ensure the user receives a totp secret
        if let TotpSettings::Enabled {
            totp_ttl: _duration,
        } = authority.settings.totp
        {
            let totp_secret_params = CreateTotpSecret {
                user_id: params.user_id,
            };

            self.totp_secrets
                .call(&totp_secret_params)
                .await?;
        }

        self.insert_user_authority
            .call((
                params.user_id,
                &user_authority,
            ))
            .await
    }
}
