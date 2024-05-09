use oxidauth_kernel::totp_secrets::select_totp_secret_by_user_id::*;
use oxidauth_repository::totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserId;
use sqlx::PgConnection;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a SelectTOTPSecrețByUserId> for Database {
    type Response = TOTPSecret;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_totp_secret_by_user_id_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a SelectTOTPSecrețByUserId,
    ) -> Result<TOTPSecret, BoxedError> {
        let mut conn = self.pool.acquire().await?;

        let result =
            select_totp_secret_by_user_id_query(&mut conn, params.user_id)
                .await?;

        Ok(role)
    }
}

pub async fn select_totp_secret_by_user_id_query(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<TOTPSecret, BoxedError> {
    let result = sqlx::query_as::<_, TOTPSecret>(include_str!(
        "./select_totp_secret_by_user_id.sql"
    ))
    .bind(user_id)
    .fetch_one(conn)
    .await?;

    Ok(result)
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
