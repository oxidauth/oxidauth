use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    totp_secrets::create_totp_secret::{
        CreateTotpSecret,
        CreateTotpSecretResponse,
        CreateTotpSecretServiceTrait,
    },
};
use oxidauth_repository::totp_secrets::insert_totp_secret::{
    InsertTotpSecretParams,
    InsertTotpSecretQuery,
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
impl<T> CreateTotpSecretServiceTrait for CreateTotpSecretUseCase<T>
where
    T: InsertTotpSecretQuery,
{
    #[tracing::instrument(name = "CreateTotpSecretUseCase::create_totp_secret", skip(self))]
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
            .insert_totp_secret(&totp_secret_params)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use uuid::Uuid;

    use super::*;

    const USER_ID: Uuid = uuid::uuid!("4d1b2c3d-4444-4000-8000-000000000001");

    #[derive(Default)]
    struct MockInsert {
        calls: Arc<Mutex<Vec<(Uuid, String)>>>,
        fail: bool,
    }

    #[async_trait]
    impl InsertTotpSecretQuery for MockInsert {
        async fn insert_totp_secret(
            &self,
            req: &InsertTotpSecretParams,
        ) -> Result<CreateTotpSecretResponse, BoxedError> {
            self.calls
                .lock()
                .expect("log")
                .push((req.user_id, req.secret_key.clone()));

            if self.fail {
                return Err("simulated insert failure".into());
            }

            Ok(CreateTotpSecretResponse { success: true })
        }
    }

    fn is_random_secret(secret: &str) -> bool {
        secret.len() == 32
            && secret
                .chars()
                .all(char::is_alphanumeric)
    }

    #[tokio::test]
    async fn generates_a_32_char_alphanumeric_secret_for_the_requested_user() {
        let mock = MockInsert::default();
        let calls = mock.calls.clone();
        let case = CreateTotpSecretUseCase::new(mock);

        let res = case
            .create_totp_secret(&CreateTotpSecret { user_id: USER_ID })
            .await
            .expect("generation must succeed");

        assert!(res.success, "the repository response is passed through");

        let calls = calls
            .lock()
            .expect("log")
            .clone();
        assert_eq!(calls.len(), 1, "exactly one insert");
        let (user_id, secret) = &calls[0];
        assert_eq!(
            *user_id, USER_ID,
            "the secret is stored for the requested user"
        );
        assert!(
            is_random_secret(secret),
            "the secret must be the 32-char alphanumeric random_string() shape, got {secret:?}"
        );
    }

    #[tokio::test]
    async fn consecutive_secrets_differ() {
        let mock = MockInsert::default();
        let calls = mock.calls.clone();
        let case = CreateTotpSecretUseCase::new(mock);

        case.create_totp_secret(&CreateTotpSecret { user_id: USER_ID })
            .await
            .expect("first");
        case.create_totp_secret(&CreateTotpSecret { user_id: USER_ID })
            .await
            .expect("second");

        let calls = calls.lock().expect("log");
        assert_ne!(
            calls[0].1, calls[1].1,
            "regenerating must not reuse the previous secret"
        );
    }

    #[tokio::test]
    async fn insert_failure_propagates() {
        let mock = MockInsert {
            fail: true,
            calls: Arc::new(Mutex::new(vec![])),
        };
        let case = CreateTotpSecretUseCase::new(mock);

        let err = case
            .create_totp_secret(&CreateTotpSecret { user_id: USER_ID })
            .await
            .expect_err("insert failures must propagate");

        assert!(
            err.to_string()
                .contains("simulated insert failure")
        );
    }
}
