use oxidauth_repository::totp_secrets::select_where_no_totp_secret_by_authority_id::{SelectWhereNoTotpSecretByAuthorityIdParams, SelectWhereNoTotpSecretByAuthorityIdQuery};
use sqlx::PgConnection;

use crate::prelude::*;

use super::*;

#[async_trait]
impl SelectWhereNoTotpSecretByAuthorityIdQuery for Database {
    #[tracing::instrument(
        name = "select_where_no_totp_secret_by_authority_id",
        skip(self)
    )]
    async fn select_where_no_totp_secret_by_authority_id(
        &self,
        params: &SelectWhereNoTotpSecretByAuthorityIdParams,
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
    let result = sqlx::query_as::<_, (Uuid,)>(include_str!(
        "./select_where_no_totp_secret_by_authority_id.sql"
    ))
    .bind(authority_id)
    .fetch_all(conn)
    .await?;

    let result = result
        .into_iter()
        .map(|row| row.0)
        .collect();

    Ok(result)
}
