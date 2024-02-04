use oxidauth_repository::users::delete_user_by_id_query::*;

use crate::prelude::*;

use super::UserRow;

#[async_trait]
impl<'a> Service<&'a DeleteUserById> for Database {
    type Response = User;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_user_by_id_query", skip(self))]
    async fn call(
        &self,
        user_id: &'a DeleteUserById,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, UserRow>(include_str!(
            "./delete_user_by_id_query.sql"
        ))
        .bind(user_id.user_id)
        .fetch_one(&self.pool)
        .await?;

        let user = result.try_into()?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    
    use sqlx::PgPool;

    

    #[ignore]
    #[sqlx::test]
    async fn it_should_delete_a_user_by_id_successfully(_pool: PgPool) {}
}
