use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    totp_secrets::create_totp_secrets_by_authority_id::{
        CreateTotpSecrets,
        CreateTotpSecretsTrait,
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
impl<S, I> CreateTotpSecretsTrait for CreateTotpSecretsByAuthorityIdUseCase<S, I>
where
    S: SelectWhereNoTotpSecretByAuthorityIdQuery,
    I: InsertTotpSecretsQuery,
{
    #[tracing::instrument(
        name = "CreateTotpSecretsByAuthorityIdUseCase::create_totp_secrets",
        skip(self)
    )]
    async fn create_totp_secrets(&self, params: &CreateTotpSecrets) -> Result<(), BoxedError> {
        let user_ids = self
            .select_user_ids
            .select_where_no_totp_secret_by_authority_id(
                &SelectWhereNoTotpSecretByAuthorityIdParams {
                    authority_id: params.authority_id,
                },
            )
            .await?;

        let user_id_and_secrets = user_ids
            .into_iter()
            .map(|user_id| (user_id, random_string()))
            .collect();

        self.insert_secrets
            .insert_totp_secrets(&InsertTotpSecretsParams {
                user_id_and_secrets,
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use uuid::Uuid;

    use super::*;

    const AUTHORITY_ID: Uuid = uuid::uuid!("5e1b2c3d-5555-4000-8000-000000000001");
    const USER_A: Uuid = uuid::uuid!("5e1b2c3d-5555-4000-8000-000000000002");
    const USER_B: Uuid = uuid::uuid!("5e1b2c3d-5555-4000-8000-000000000003");

    #[derive(Default)]
    struct MockSelect {
        user_ids: Vec<Uuid>,
        fail: bool,
        seen: Arc<Mutex<Vec<Uuid>>>,
    }

    #[async_trait]
    impl SelectWhereNoTotpSecretByAuthorityIdQuery for MockSelect {
        async fn select_where_no_totp_secret_by_authority_id(
            &self,
            params: &SelectWhereNoTotpSecretByAuthorityIdParams,
        ) -> Result<Vec<Uuid>, BoxedError> {
            self.seen
                .lock()
                .expect("log")
                .push(params.authority_id);

            if self.fail {
                return Err("simulated select failure".into());
            }

            Ok(self.user_ids.clone())
        }
    }

    #[derive(Default)]
    struct MockInsert {
        fail: bool,
        batches: Arc<Mutex<Vec<Vec<(Uuid, String)>>>>,
    }

    #[async_trait]
    impl InsertTotpSecretsQuery for MockInsert {
        async fn insert_totp_secrets(
            &self,
            params: &InsertTotpSecretsParams,
        ) -> Result<(), BoxedError> {
            self.batches
                .lock()
                .expect("log")
                .push(
                    params
                        .user_id_and_secrets
                        .clone(),
                );

            if self.fail {
                return Err("simulated batch insert failure".into());
            }

            Ok(())
        }
    }

    fn use_case(
        select: MockSelect,
        insert: MockInsert,
    ) -> CreateTotpSecretsByAuthorityIdUseCase<MockSelect, MockInsert> {
        CreateTotpSecretsByAuthorityIdUseCase::new(select, insert)
    }

    #[tokio::test]
    async fn mints_one_random_secret_per_user_lacking_one() {
        let seen = Arc::new(Mutex::new(vec![]));
        let batches = Arc::new(Mutex::new(vec![]));

        let case = use_case(
            MockSelect {
                user_ids: vec![USER_A, USER_B],
                fail: false,
                seen: seen.clone(),
            },
            MockInsert {
                fail: false,
                batches: batches.clone(),
            },
        );

        case.create_totp_secrets(&CreateTotpSecrets {
            authority_id: AUTHORITY_ID,
        })
        .await
        .expect("the batch rollout must succeed");

        assert_eq!(
            seen.lock()
                .expect("log")
                .as_slice(),
            &[AUTHORITY_ID],
            "the anti-join query runs against the requested authority"
        );

        let batches = batches.lock().expect("log");
        assert_eq!(batches.len(), 1, "one batch insert");
        let batch = &batches[0];
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].0, USER_A);
        assert_eq!(batch[1].0, USER_B, "input user order is preserved");
        for (_, secret) in batch {
            assert!(
                secret.len() == 32
                    && secret
                        .chars()
                        .all(char::is_alphanumeric),
                "each secret must be the 32-char alphanumeric random_string() shape, got {secret:?}"
            );
        }
        assert_ne!(
            batch[0].1, batch[1].1,
            "secrets must not repeat across users"
        );
    }

    #[tokio::test]
    async fn empty_anti_join_still_runs_an_empty_batch_insert() {
        let seen = Arc::new(Mutex::new(vec![]));
        let batches = Arc::new(Mutex::new(vec![]));

        let case = use_case(
            MockSelect {
                user_ids: vec![],
                fail: false,
                seen,
            },
            MockInsert {
                fail: false,
                batches: batches.clone(),
            },
        );

        case.create_totp_secrets(&CreateTotpSecrets {
            authority_id: AUTHORITY_ID,
        })
        .await
        .expect("an empty rollout is a success");

        let batches = batches.lock().expect("log");
        assert_eq!(
            batches.as_slice(),
            &[vec![]],
            "pinned: the insert runs even with zero users (no short-circuit)"
        );
    }

    #[tokio::test]
    async fn select_failure_prevents_any_insert() {
        let batches = Arc::new(Mutex::new(vec![]));

        let case = use_case(
            MockSelect {
                user_ids: vec![],
                fail: true,
                seen: Arc::new(Mutex::new(vec![])),
            },
            MockInsert {
                fail: false,
                batches: batches.clone(),
            },
        );

        let err = case
            .create_totp_secrets(&CreateTotpSecrets {
                authority_id: AUTHORITY_ID,
            })
            .await
            .expect_err("select failures must propagate");

        assert!(
            err.to_string()
                .contains("simulated select failure")
        );
        assert!(
            batches
                .lock()
                .expect("log")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn insert_failure_propagates() {
        let case = use_case(
            MockSelect {
                user_ids: vec![USER_A],
                fail: false,
                seen: Arc::new(Mutex::new(vec![])),
            },
            MockInsert {
                fail: true,
                batches: Arc::new(Mutex::new(vec![])),
            },
        );

        let err = case
            .create_totp_secrets(&CreateTotpSecrets {
                authority_id: AUTHORITY_ID,
            })
            .await
            .expect_err("insert failures must propagate");

        assert!(
            err.to_string()
                .contains("simulated batch insert failure")
        );
    }
}
