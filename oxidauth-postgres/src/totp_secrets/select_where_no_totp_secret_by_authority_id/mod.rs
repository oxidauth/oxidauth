use oxidauth_kernel::totp_secrets::{
    find_totp_secret_by_user_id::*, TOTPSecret,
};
use sqlx::PgConnection;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a SelectWhereNoTotpSecretByAuthorityIdParams> for Database {
    type Response = Vec<Uuid>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_where_no_totp_secret_by_authority_id",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a SelectWhereNoTotpSecretByAuthorityIdParams,
    ) -> Result<Vec<Uuid>, BoxedError> {
        let mut conn = self.pool.acquire().await?;

        select_where_no_totp_secret_by_authority_id(
            &mut conn,
            params.authority_id,
        )
        .await
    }
}

pub async fn select_where_no_totp_secret_by_authority_id(
    conn: &mut PgConnection,
    authority_id: Uuid,
) -> Result<Vec<Uuid>, BoxedError> {
    let result = sqlx::query_as::<_, Vec<Uuid>>(include_str!(
        "./select_where_no_totp_secret_by_authority_id.sql"
    ))
    .bind(authority_id)
    .fetch_one(conn)
    .await?;

    Ok(result)
}
