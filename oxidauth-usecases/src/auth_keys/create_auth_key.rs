use async_trait::async_trait;
use oxidauth_kernel::{
    auth_keys::create_auth_key::{
        CreateAuthKey, CreateAuthKeyResponse, InsertAuthKeyParams,
    },
    error::BoxedError,
    service::Service,
};
use oxidauth_repository::auth_keys::insert_auth_keys::InsertAuthKeyQuery;

use rand::prelude::*;

pub struct CreateAuthKeyUseCase<T>
where
    T: InsertAuthKeyQuery,
{
    auth_keys: T,
}

impl<T> CreateAuthKeyUseCase<T>
where
    T: InsertAuthKeyQuery,
{
    pub fn new(auth_keys: T) -> Self {
        Self { auth_keys }
    }
}

#[async_trait]
impl<'a, T> Service<&'a mut CreateAuthKey> for CreateAuthKeyUseCase<T>
where
    T: InsertAuthKeyQuery,
{
    type Response = CreateAuthKeyResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "create_auth_key_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a mut CreateAuthKey,
    ) -> Result<Self::Response, Self::Error> {
        let nums = generate_secret();

        let totp_secret_params = InsertAuthKeyParams {
            user_id: req.user_id,
            secret_key: nums,
        };

        self.auth_keys
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
