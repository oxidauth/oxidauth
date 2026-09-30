use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, public_keys::find_public_key_by_id::*};
use oxidauth_repository::public_keys::select_public_key_by_id::SelectPublicKeyByIdQuery;

pub struct FindPublicKeyByIdUseCase<T>
where
    T: SelectPublicKeyByIdQuery,
{
    public_keys: T,
}

impl<T> FindPublicKeyByIdUseCase<T>
where
    T: SelectPublicKeyByIdQuery,
{
    pub fn new(public_keys: T) -> Self {
        Self { public_keys }
    }
}

#[async_trait]
impl<T> FindPublicKeyByIdServiceTrait for FindPublicKeyByIdUseCase<T>
where
    T: SelectPublicKeyByIdQuery,
{
    #[tracing::instrument(name = "FindPublicKeyByIdUseCase::find_public_key_by_id", skip(self))]
    async fn find_public_key_by_id(
        &self,
        req: &FindPublicKeyById,
    ) -> Result<PublicKey, BoxedError> {
        self.public_keys
            .select_public_key_by_id(req)
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

    fn stored_key() -> PublicKey {
        PublicKey {
            id: key_id(),
            // rows store base64(PEM); unlike `list_all_public_keys`, find
            // does not decode
            public_key: "LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0=".to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Default)]
    struct Log {
        calls: Vec<Uuid>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn calls(log: &SharedLog) -> Vec<Uuid> {
        log.lock()
            .expect("log")
            .calls
            .clone()
    }

    struct MockPublicKeys {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl SelectPublicKeyByIdQuery for MockPublicKeys {
        async fn select_public_key_by_id(
            &self,
            req: &FindPublicKeyById,
        ) -> Result<PublicKey, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .calls
                .push(req.public_key_id);

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(stored_key())
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> FindPublicKeyByIdUseCase<MockPublicKeys> {
        FindPublicKeyByIdUseCase::new(MockPublicKeys {
            log: log.clone(),
            fail,
        })
    }

    // Pure delegate — base64 storage reaches callers untouched.
    #[tokio::test]
    async fn id_reaches_the_query_and_the_row_is_returned_undecoded() {
        let log = Arc::new(Mutex::new(Log::default()));

        let key = use_case(&log, false)
            .find_public_key_by_id(&FindPublicKeyById {
                public_key_id: key_id(),
            })
            .await
            .expect("find succeeds");

        assert_eq!(calls(&log), vec![key_id()]);
        assert_eq!(key.public_key, stored_key().public_key);
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .find_public_key_by_id(&FindPublicKeyById {
                public_key_id: key_id(),
            })
            .await
            .expect_err("RowNotFound from the query propagates unchanged");

        assert_eq!(error.to_string(), "simulated select failure");
    }
}
