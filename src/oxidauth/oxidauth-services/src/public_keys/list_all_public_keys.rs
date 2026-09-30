use async_trait::async_trait;
use base64::prelude::*;
use oxidauth_kernel::{error::BoxedError, public_keys::list_all_public_keys::*};
use oxidauth_repository::public_keys::select_all_public_keys::SelectAllPublicKeysQuery;

pub struct ListAllPublicKeysUseCase<T>
where
    T: SelectAllPublicKeysQuery,
{
    public_keys: T,
}

impl<T> ListAllPublicKeysUseCase<T>
where
    T: SelectAllPublicKeysQuery,
{
    pub fn new(public_keys: T) -> Self {
        Self { public_keys }
    }
}

#[async_trait]
impl<T> ListAllPublicKeysServiceTrait for ListAllPublicKeysUseCase<T>
where
    T: SelectAllPublicKeysQuery,
{
    #[tracing::instrument(name = "ListAllPublicKeysUseCase::list_all_public_keys", skip(self))]
    async fn list_all_public_keys(
        &self,
        req: &ListAllPublicKeys,
    ) -> Result<Vec<PublicKey>, BoxedError> {
        let public_keys = self
            .public_keys
            .select_all_public_keys(req)
            .await?;

        // rows store base64(PEM) — the jwks payload needs the raw PEM.
        // Rows written outside `create_public_key` (manual seeds, restores)
        // can break that invariant: skip any row that is not standard base64
        // or does not decode to UTF-8, logging its id, instead of `unwrap()`
        // panicking the whole listing. This use case also runs inside
        // `ExtractJwt`, so one panic here once took down every authenticated
        // request; a malformed row can verify nothing anyway, so dropping it
        // loses no capability (precedent: the `oxidauth-rs` client skips the
        // same rows, `client/mod.rs`).
        let mut keys = Vec::with_capacity(public_keys.len());

        for mut pk in public_keys {
            let decoded = match BASE64_STANDARD.decode(&pk.public_key) {
                Ok(decoded) => decoded,
                Err(err) => {
                    tracing::error!(
                        key_id = %pk.id,
                        err = %err,
                        "public_keys row is not standard base64; excluding from jwks listing"
                    );

                    continue;
                },
            };

            match String::from_utf8(decoded) {
                Ok(pem) => {
                    pk.public_key = pem;
                    keys.push(pk);
                },
                Err(err) => {
                    tracing::error!(
                        key_id = %pk.id,
                        err = %err,
                        "public_keys row decodes to non-UTF8 bytes; excluding from jwks listing"
                    );
                },
            }
        }

        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use base64::prelude::*;
    use chrono::Utc;

    use super::*;

    const PEM: &str = "-----BEGIN PUBLIC KEY-----\nMIIB-fake\n-----END PUBLIC KEY-----\n";

    fn key(public_key: &str) -> PublicKey {
        PublicKey {
            id: uuid::uuid!("11111111-1111-4111-8111-111111111111"),
            public_key: public_key.to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Default)]
    struct Log {
        calls: usize,
    }

    type SharedLog = Arc<Mutex<Log>>;

    struct MockPublicKeys {
        log: SharedLog,
        rows: Vec<String>,
        fail: bool,
    }

    #[async_trait]
    impl SelectAllPublicKeysQuery for MockPublicKeys {
        async fn select_all_public_keys(
            &self,
            params: &ListAllPublicKeys,
        ) -> Result<Vec<PublicKey>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .calls += 1;

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(self
                .rows
                .iter()
                .map(|pk| key(pk))
                .collect())
        }
    }

    fn use_case(
        log: &SharedLog,
        rows: Vec<String>,
        fail: bool,
    ) -> ListAllPublicKeysUseCase<MockPublicKeys> {
        ListAllPublicKeysUseCase::new(MockPublicKeys {
            log: log.clone(),
            rows,
            fail,
        })
    }

    #[tokio::test]
    async fn stored_base64_material_is_decoded_back_to_pem() {
        // the jwks payload is minted here: rows store base64(PEM) (what
        // `create_public_key` writes) and the service unwraps one layer
        let log = Arc::new(Mutex::new(Log::default()));
        let stored = BASE64_STANDARD.encode(PEM);

        let keys = use_case(&log, vec![stored.clone()], false)
            .list_all_public_keys(&ListAllPublicKeys)
            .await
            .expect("list succeeds");

        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].public_key, PEM);
        assert_eq!(keys[0].id, key("").id);
    }

    #[tokio::test]
    async fn an_empty_keyset_passes_through() {
        let log = Arc::new(Mutex::new(Log::default()));

        let keys = use_case(&log, vec![], false)
            .list_all_public_keys(&ListAllPublicKeys)
            .await
            .expect("list succeeds");

        assert!(keys.is_empty());
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, vec!["irrelevant".to_owned()], true)
            .list_all_public_keys(&ListAllPublicKeys)
            .await
            .expect_err("select failure propagates");

        assert_eq!(error.to_string(), "simulated select failure");
    }

    #[tokio::test]
    async fn a_row_whose_public_key_is_not_base64_is_skipped() {
        // OXA-000012 fix: a `public_key` written raw outside
        // `create_public_key` (a plain PEM — the armor is not standard
        // base64) is skipped with an error log instead of panicking the
        // whole listing. The repository query is still hit exactly once.
        let log = Arc::new(Mutex::new(Log::default()));

        let keys = use_case(&log, vec![PEM.to_owned()], false)
            .list_all_public_keys(&ListAllPublicKeys)
            .await
            .expect("listing survives the malformed row");

        assert!(keys.is_empty());
        assert_eq!(log.lock().expect("log").calls, 1);
    }

    #[tokio::test]
    async fn valid_base64_that_is_not_utf8_is_skipped() {
        // the other leg: base64 of arbitrary bytes decodes fine but is not
        // UTF-8; the row is excluded from an otherwise-successful listing
        let log = Arc::new(Mutex::new(Log::default()));
        let stored = BASE64_STANDARD.encode([0xFF_u8, 0xFE, 0x00]);

        let keys = use_case(&log, vec![stored], false)
            .list_all_public_keys(&ListAllPublicKeys)
            .await
            .expect("listing survives the malformed row");

        assert!(keys.is_empty());
        assert_eq!(log.lock().expect("log").calls, 1);
    }

    #[tokio::test]
    async fn a_mixed_set_serves_the_healthy_key_and_skips_corrupt_rows() {
        // the availability invariant `ExtractJwt` depends on: rows corrupt
        // in either leg are excluded and every healthy key keeps serving
        let log = Arc::new(Mutex::new(Log::default()));
        let healthy = BASE64_STANDARD.encode(PEM);

        let keys = use_case(
            &log,
            vec![
                healthy,
                PEM.to_owned(),
                BASE64_STANDARD.encode([0xFF_u8, 0xFE, 0x00]),
            ],
            false,
        )
        .list_all_public_keys(&ListAllPublicKeys)
        .await
        .expect("listing survives the malformed rows");

        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].public_key, PEM);
        assert_eq!(log.lock().expect("log").calls, 1);
    }
}
