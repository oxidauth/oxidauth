use oxidauth_kernel::user_role_grants::{
    list_user_role_grants_by_user_id::ListUserRoleGrantsByUserId, UserRole,
};

use crate::prelude::*;

use super::PgUserRole;

#[async_trait]
impl<'a> Service<&'a ListUserRoleGrantsByUserId> for Database {
    type Response = Vec<UserRole>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_user_role_grants_by_user_id_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a ListUserRoleGrantsByUserId,
    ) -> Result<Self::Response, Self::Error> {
        let pool = self.read_pool();
        let mut conn = pool.acquire().await?;

        let user_role_grants =
            select_user_role_grants_by_user_id_query(&mut conn, params.user_id)
                .await?
                .into_iter()
                .map(Into::into)
                .collect();

        Ok(user_role_grants)
    }
}

pub async fn select_user_role_grants_by_user_id_query(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<PgUserRole>, BoxedError> {
    let user_role_grants = sqlx::query_as::<_, PgUserRole>(include_str!(
        "./select_user_role_grants_by_user_id_query.sql"
    ))
    .bind(user_id)
    .fetch_all(conn)
    .await?;

    Ok(user_role_grants)
}



