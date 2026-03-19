use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    totp_secrets::create_totp_secret::{
        CreateTotpSecret, CreateTotpSecretResponse, CreateTotpSecretTrait,
    },
};
use oxidauth_repository::totp_secrets::insert_totp_secret::{
    InsertTotpSecretParams, InsertTotpSecretQuery,
};

use crate::random_string;

pub struct CreateTotpSecretUseCase<T>
where
    T: InsertTotpSecretQuery,
{
    totp_secrets: T,
}

impl<T> CreateTotpSecretUseCase<T>
where
    T: InsertTotpSecretQuery,
{
    pub fn new(totp_secrets: T) -> Self {
        Self { totp_secrets }
    }
}

#[async_trait]
impl<T> CreateTotpSecretTrait for CreateTotpSecretUseCase<T>
where
    T: InsertTotpSecretQuery,
{
    #[tracing::instrument(name = "create_totp_secret_usecase", skip(self))]
    async fn create_totp_secret(
        &self,
        req: &CreateTotpSecret,
    ) -> Result<CreateTotpSecretResponse, BoxedError> {
        let nums = random_string();

        let totp_secret_params = InsertTotpSecretParams {
            user_id: req.user_id,
            secret_key: nums,
        };

        self.totp_secrets
            .call(&totp_secret_params)
            .await
    }
}
