use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    totp_secrets::create_totp_secrets_by_authority_id::{
        CreateTotpSecrets, CreateTotpSecretsTrait,
    },
};
use oxidauth_repository::totp_secrets::{
    insert_totp_secrets::{InsertTotpSecretsParams, InsertTotpSecretsQuery},
    select_where_no_totp_secret_by_authority_id::{
        SelectWhereNoTotpSecretByAuthorityIdParams,
        SelectWhereNoTotpSecretByAuthorityIdQuery,
    },
};

use crate::random_string;

#[derive(Debug, Clone)]
pub struct CreateTotpSecretsByAuthorityIdUseCase<S, I>
where
    S: SelectWhereNoTotpSecretByAuthorityIdQuery,
    I: InsertTotpSecretsQuery,
{
    select_user_ids: S,
    insert_secrets: I,
}

impl<S, I> CreateTotpSecretsByAuthorityIdUseCase<S, I>
where
    S: SelectWhereNoTotpSecretByAuthorityIdQuery,
    I: InsertTotpSecretsQuery,
{
    pub fn new(select_user_ids: S, insert_secrets: I) -> Self {
        Self {
            select_user_ids,
            insert_secrets,
        }
    }
}

#[async_trait]
impl<S, I> CreateTotpSecretsTrait
    for CreateTotpSecretsByAuthorityIdUseCase<S, I>
where
    S: SelectWhereNoTotpSecretByAuthorityIdQuery,
    I: InsertTotpSecretsQuery,
{
    #[tracing::instrument(
        name = "create_totp_secrets_by_authority_id_usecase",
        skip(self)
    )]
    async fn create_totp_secrets(
        &self,
        req: &CreateTotpSecrets,
    ) -> Result<(), BoxedError> {
        // get the user ids via the authority
        let user_ids = self
            .select_user_ids
            .select_where_no_totp_secret_by_authority_id(
                &SelectWhereNoTotpSecretByAuthorityIdParams {
                    authority_id: req.authority_id,
                },
            )
            .await?;

        let user_id_and_secrets: Vec<_> = user_ids
            .into_iter()
            .map(|user_id| (user_id, random_string()))
            .collect();

        self.insert_secrets
            .insert_totp_secrets(&InsertTotpSecretsParams {
                user_id_and_secrets,
            })
            .await?;

        Ok(())
    }
}
