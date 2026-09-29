use oxidauth_repository::users::select_all_users_query::*;

use crate::prelude::*;

use super::{TryFromUserRowError, UserRow};

#[async_trait]
impl<'a> Service<&'a ListAllUsers> for Database {
    type Response = Vec<User>;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_all_users_query", skip(self))]
    async fn call(
        &self,
        _params: &'a ListAllUsers,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, UserRow>(include_str!(
            "./select_all_users_query.sql"
        ))
        .fetch_all(&self.pool)
        .await?;

        let users = result
            .into_iter()
            .map(|u| u.try_into())
            .collect::<Result<Vec<User>, TryFromUserRowError>>()?;

        Ok(users)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_users_successfully(_pool: PgPool) {}
}
