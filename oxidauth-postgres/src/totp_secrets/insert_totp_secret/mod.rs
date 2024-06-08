use crate::prelude::*;

use oxidauth_kernel::totp_secrets::create_totp_secret::CreateTotpSecretResponse;
use oxidauth_repository::totp_secrets::insert_totp_secret::InsertTotpSecretParams;

use super::{PgTotpSecret, TOTPSecretRow};

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
        let mut conn = self.pool.acquire().await?;

        let result = insert_totp_secret_query(&mut conn, params).await?;

        let _: TOTPSecretRow = result.try_into()?;

        let response = CreateTotpSecretResponse { success: true };

        Ok(response)
    }
}

pub async fn insert_totp_secret_query(
    conn: &mut PgConnection,
    params: &InsertTotpSecretParams,
) -> Result<PgTotpSecret, BoxedError> {
    let result = sqlx::query_as::<_, PgTotpSecret>(include_str!(
        "./insert_totp_secret.sql"
    ))
    .bind(&params.user_id)
    .bind(&params.secret_key)
    .fetch_one(conn)
    .await
    .map_err(|err| Box::new(err))?;

    Ok(result)
}
