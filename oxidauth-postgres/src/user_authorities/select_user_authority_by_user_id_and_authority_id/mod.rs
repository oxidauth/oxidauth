use oxidauth_repository::user_authorities::select_user_authority_by_user_id_and_authority_id::*;

use crate::prelude::*;

use super::PgUserAuthorityWithAuthority;

#[async_trait]
impl<'a> Service<&'a FindUserAuthorityByUserIdAndAuthorityId> for Database {
    type Response = UserAuthorityWithAuthority;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_user_authority_by_user_id_and_authority_id_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a FindUserAuthorityByUserIdAndAuthorityId,
    ) -> Result<Self::Response, Self::Error> {
        let result =
            sqlx::query_as::<_, PgUserAuthorityWithAuthority>(include_str!(
                "./select_user_authority_by_user_id_and_authority_id.sql"
            ))
            .bind(params.user_id)
            .bind(params.authority_id)
            .fetch_one(&self.pool)
            .await?;

        let user = result.try_into()?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::users::insert_user::*;
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_a_user_authority_by_ids_successfully(
        pool: PgPool,
    ) {
    }
}
