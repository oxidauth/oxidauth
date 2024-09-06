use crate::{
    prelude::*, totp_secrets::insert_totp_secret::insert_totp_secret_query,
};

use oxidauth_repository::totp_secrets::{
    insert_totp_secret::InsertTotpSecretParams,
    insert_totp_secrets::{InsertTotpSecretsParams, InsertTotpSecretsQuery},
};

#[async_trait]
impl InsertTotpSecretsQuery for Database {
    #[tracing::instrument(
        name = "insert_totp_secrets_query",
        skip(self, params)
    )]
    async fn insert_totp_secrets(
        &self,
        params: &InsertTotpSecretsParams,
    ) -> Result<(), BoxedError> {
        let mut tx = self.pool.begin().await?;

        for (user_id, secret_key) in params
            .user_id_and_secrets
            .clone()
            .into_iter()
        {
            insert_totp_secret_query(
                &mut tx,
                &InsertTotpSecretParams {
                    user_id,
                    secret_key,
                },
            )
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
