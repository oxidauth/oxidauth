use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, public_keys::delete_public_key::*};
use oxidauth_repository::public_keys::delete_public_key::DeletePublicKeyQuery;

pub struct DeletePublicKeyUseCase<T>
where
    T: DeletePublicKeyQuery,
{
    public_keys: T,
}

impl<T> DeletePublicKeyUseCase<T>
where
    T: DeletePublicKeyQuery,
{
    pub fn new(public_keys: T) -> Self {
        Self { public_keys }
    }
}

#[async_trait]
impl<T> DeletePublicKeyServiceTrait for DeletePublicKeyUseCase<T>
where
    T: DeletePublicKeyQuery,
{
    #[tracing::instrument(name = "DeletePublicKeyUseCase::delete_public_key", skip(self))]
    async fn delete_public_key(&self, req: &DeletePublicKey) -> Result<PublicKey, BoxedError> {
        self.public_keys
            .delete_public_key(req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use uuid::Uuid;

    use super::*;

    fn key_id() -> Uuid {
        uuid::uuid!("11111111-1111-4111-8111-111111111111")
    }

    fn deleted_key() -> PublicKey {
        PublicKey {
            id: key_id(),
            public_key: "-----BEGIN PUBLIC KEY-----".to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Default)]
    struct Log {
        deletes: Vec<Uuid>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn deletes(log: &SharedLog) -> Vec<Uuid> {
        log.lock()
            .expect("log")
            .deletes
            .clone()
    }

    struct MockPublicKeys {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl DeletePublicKeyQuery for MockPublicKeys {
        async fn delete_public_key(&self, req: &DeletePublicKey) -> Result<PublicKey, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .deletes
                .push(req.public_key_id);

            if self.fail {
                return Err("simulated delete failure".into());
            }

            Ok(deleted_key())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> DeletePublicKeyUseCase<MockPublicKeys> {
        DeletePublicKeyUseCase::new(MockPublicKeys {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate.
    #[tokio::test]
    async fn id_reaches_the_query_and_the_deleted_row_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));

        let key = use_case(&log, false)
            .delete_public_key(&DeletePublicKey {
                public_key_id: key_id(),
            })
            .await
            .expect("delete succeeds");

        assert_eq!(deletes(&log), vec![key_id()]);
        assert_eq!(key.id, key_id());
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        // a missing id raises `RowNotFound` inside the query — the service
        // relays it without remapping
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .delete_public_key(&DeletePublicKey {
                public_key_id: key_id(),
            })
            .await
            .expect_err("delete failure propagates");

        assert_eq!(error.to_string(), "simulated delete failure");
    }
}
