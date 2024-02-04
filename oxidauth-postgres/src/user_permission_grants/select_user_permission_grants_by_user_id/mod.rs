use oxidauth_kernel::user_permission_grants::{
    list_user_permission_grants_by_user_id::ListUserPermissionGrantsByUserId,
    UserPermission,
};

use crate::prelude::*;

use super::PgUserPermission;

#[async_trait]
impl<'a> Service<&'a ListUserPermissionGrantsByUserId> for Database {
    type Response = Vec<UserPermission>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_user_permission_grants_by_user_id_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a ListUserPermissionGrantsByUserId,
    ) -> Result<Self::Response, Self::Error> {
        let mut conn = self.pool.acquire().await?;

        let user_permission_grants =
            select_user_permission_grants_by_user_id_query(&mut conn, params.user_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(user_permission_grants)
    }
}

pub async fn select_user_permission_grants_by_user_id_query(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<PgUserPermission>, BoxedError> {
    let user_permission_grants = sqlx::query_as::<_, PgUserPermission>(
        include_str!("./select_user_permission_grants_by_user_id_query.sql"),
    )
    .bind(user_id)
    .fetch_all(conn)
    .await?;

    Ok(user_permission_grants)
}

#[cfg(test)]
mod tests {
    
    use sqlx::PgPool;

    

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_user_permission_grants_successfully(_pool: PgPool) {
    }
}
