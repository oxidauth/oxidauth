use crate::prelude::*;

use oxidauth_kernel::auth_keys::create_auth_key::{
    AuthKey, CreateAuthKeyResponse, InsertAuthKeyParams,
};

use super::PgAuthKey;

#[async_trait]
impl<'a> Service<&'a InsertAuthKeyParams> for Database {
    type Response = CreateAuthKeyResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "insert_auth_key_query", skip(self, params))]
    async fn call(
        &self,
        params: &'a InsertAuthKeyParams,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, PgAuthKey>(include_str!(
            "./insert_auth_key.sql"
        ))
        .bind(&params.user_id)
        .bind(&params.secret_key)
        .fetch_one(&self.pool)
        .await?;

        let public_key: AuthKey = result.try_into()?;

        let response = CreateAuthKeyResponse {
            user_id: public_key.user_id,
        };

        Ok(response)
    }
}
