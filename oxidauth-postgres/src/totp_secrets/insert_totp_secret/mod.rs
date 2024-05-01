use crate::prelude::*;

use oxidauth_kernel::totp_secrets::create_totp_secret::{
    CreateTotpSecretResponse, InsertTotpSecretParams, TotpSecret,
};

use super::PgTotpSecret;

#[async_trait]
impl<'a> Service<&'a InsertTotpSecretParams> for Database {
    type Response = CreateTotpSecretResponse;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "insert_totp_secret_query",
        skip(self, params)
    )]
    async fn call(
        &self,
        params: &'a InsertTotpSecretParams,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, PgTotpSecret>(include_str!(
            "./insert_totp_secret.sql"
        ))
        .bind(&params.user_id)
        .bind(&params.secret_key)
        .fetch_one(&self.pool)
        .await?;

        let public_key: TotpSecret = result.try_into()?;

        let response = CreateTotpSecretResponse {
            user_id: public_key.user_id,
        };

        Ok(response)
    }
}
