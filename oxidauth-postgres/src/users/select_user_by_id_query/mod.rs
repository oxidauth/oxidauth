use oxidauth_repository::users::select_user_by_id_query::*;
use sqlx::PgConnection;

use crate::prelude::*;

use super::UserRow;

#[async_trait]
impl<'a> Service<&'a FindUserById> for Database {
    type Response = User;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_user_by_id_query", skip(self))]
    async fn call(
        &self,
        user_id: &'a FindUserById,
    ) -> Result<Self::Response, Self::Error> {
        let mut conn = self.pool.acquire().await?;

        let result =
            select_user_by_id_query(&mut conn, user_id.user_id).await?;

        let user = result.try_into()?;

        Ok(user)
    }
}

pub async fn select_user_by_id_query(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<UserRow, BoxedError> {
    let result = sqlx::query_as::<_, UserRow>(include_str!(
        "./select_user_by_id_query.sql"
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
    async fn it_should_query_a_user_by_id_successfully(_pool: PgPool) {}
}
