use oxidauth_kernel::totp_secrets::{
    find_totp_secret_by_user_id::*, TOTPSecret,
};
use sqlx::PgConnection;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a FindTOTPSecretByUserId> for Database {
    type Response = TOTPSecret;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_totp_secret_by_user_id_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a FindTOTPSecretByUserId,
    ) -> Result<TOTPSecret, BoxedError> {
        let mut conn = self.pool.acquire().await?;

        let result =
            select_totp_secret_by_user_id_query(&mut conn, params.user_id)
                .await?;

        let secret = TOTPSecret {
            secret: result.totp_secret,
        };

        Ok(secret)
    }
}

pub async fn select_totp_secret_by_user_id_query(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<TOTPSecretRow, BoxedError> {
    let result = sqlx::query_as::<_, PgTotpSecret>(include_str!(
        "./select_totp_secret_by_user_id.sql"
    ))
        .bind(user_id)
        .fetch_one(conn)
        .await?;

    Ok(result.try_into()?)
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_a_totp_secret_by_user_id_successfully(
        _pool: PgPool,
    ) {
    }
}
