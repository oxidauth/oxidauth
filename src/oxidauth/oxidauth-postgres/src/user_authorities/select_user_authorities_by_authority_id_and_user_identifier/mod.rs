use oxidauth_kernel::user_authorities::UserAuthority;
use oxidauth_repository::user_authorities::select_user_authorities_by_authority_id_and_user_identifier::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams> for Database {
    type Response = UserAuthority;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_user_authority_by_user_id_and_user_identifier_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, PgUserAuthority>(include_str!(
            "./select_user_authorities_by_authority_id_and_user_identifier.sql"
        ))
        .bind(params.authority_id)
        .bind(&params.user_identifier)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => format!("user authority not found: {:?}", err),
            _ => err.to_string(),
        })?;

        let user_authority = result.into();

        Ok(user_authority)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_a_user_authority_by_authority_id_and_user_identifier_successfully(
        _pool: PgPool,
    ) {
    }
}
