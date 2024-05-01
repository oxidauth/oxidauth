use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    service::Service,
    totp_secrets::create_totp_secret::{
        CreateTotpSecret, CreateTotpSecretResponse, InsertTotpSecretParams,
    },
};
use oxidauth_repository::totp_secrets::insert_totp_secret::InsertTotpSecretQuery;

use rand::prelude::*;

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
impl<'a, T> Service<&'a CreateTotpSecret> for CreateTotpSecretUseCase<T>
where
    T: InsertTotpSecretQuery,
{
    type Response = CreateTotpSecretResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "create_totp_secret_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a CreateTotpSecret,
    ) -> Result<Self::Response, Self::Error> {
        let nums = generate_secret();

        let totp_secret_params = InsertTotpSecretParams {
            user_id: req.user_id,
            secret_key: nums,
        };

        self.totp_secrets
            .call(&totp_secret_params)
            .await
    }
}

fn generate_secret() -> Vec<i32> {
    let mut nums: Vec<i32> = (1..100).collect();

    let mut rng: ThreadRng = rand::thread_rng();

    nums.shuffle(&mut rng);

    nums
}
