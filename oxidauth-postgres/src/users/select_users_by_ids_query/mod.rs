use oxidauth_repository::users::select_users_by_ids_query::*;
use sqlx::PgConnection;

use crate::prelude::*;

use super::{TryFromUserRowError, UserRow};

#[async_trait]
impl<'a> Service<&'a FindUsersByIds> for Database {
    type Response = (Vec<User>, Vec<Uuid>);
    type Error = BoxedError;

    #[tracing::instrument(name = "select_users_by_ids_query", skip(self))]
    async fn call(
        &self,
        params: &'a FindUsersByIds,
    ) -> Result<Self::Response, Self::Error> {
        let mut conn = self.pool.acquire().await?;

        let user_rows =
            select_users_by_ids_query(&mut conn, &params.user_ids).await?;

        let user_ids_found: Vec<Uuid> = user_rows
            .clone()
            .into_iter()
            .map(|u| u.id)
            .collect();

        let mut user_ids_not_found = params.user_ids.clone();

        user_ids_not_found.retain(|id| !&user_ids_found.contains(id));

        let users = user_rows
            .into_iter()
            .map(|u| {
                u.try_into()
                    .map_err(|err: TryFromUserRowError| Box::new(err).into())
            })
            .collect::<Result<Vec<User>, BoxedError>>()?;

        Ok((users, user_ids_not_found))
    }
}

pub async fn select_users_by_ids_query(
    conn: &mut PgConnection,
    user_ids: &[Uuid],
) -> Result<Vec<UserRow>, BoxedError> {
    let result = sqlx::query_as::<_, UserRow>(include_str!(
        "./select_users_by_ids_query.sql"
    ))
    .bind(user_ids)
    .fetch_all(conn)
    .await?;

    Ok(result)
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_users_by_ids_successfully(_pool: PgPool) {}
}
