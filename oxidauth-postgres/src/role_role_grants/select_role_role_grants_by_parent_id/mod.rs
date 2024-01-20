use oxidauth_kernel::role_role_grants::list_role_role_grants_by_parent_id::*;
use oxidauth_repository::role_role_grants::select_role_role_grants_by_parent_id::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a ListRoleRoleGrantsByParentId> for Database {
    type Response = Vec<RoleRoleGrantDetail>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_role_role_grants_by_parent_id_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a ListRoleRoleGrantsByParentId,
    ) -> Result<Vec<RoleRoleGrantDetail>, BoxedError> {
        let mut conn = self.pool.acquire().await?;

        let result = select_role_role_grants_by_parent_id_query(&mut conn, params.parent_id).await?;

        let role_role_grant = result
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(role_role_grant)
    }
}

pub async fn select_role_role_grants_by_parent_id_query(
    conn: &mut PgConnection,
    role_id: Uuid,
) -> Result<Vec<PgRoleRoleGrantDetail>, BoxedError> {
    let result = sqlx::query_as::<_, PgRoleRoleGrantDetail>(include_str!(
        "./select_role_role_grants_by_parent_id.sql"
    ))
    .bind(role_id)
    .fetch_all(conn)
    .await?;

    Ok(result)
}
