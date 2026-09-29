use oxidauth_kernel::role_permission_grants::list_role_permission_grants_by_role_id::*;
use oxidauth_repository::role_permission_grants::select_role_permission_grants_by_role_id::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a ListRolePermissionGrantsByRoleId> for Database {
    type Response = Vec<RolePermission>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_role_permission_grants_by_role_id_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a ListRolePermissionGrantsByRoleId,
    ) -> Result<Vec<RolePermission>, BoxedError> {
        let mut conn = self.pool.acquire().await?;

        let result = select_role_permission_grants_by_role_id_query(
            &mut conn,
            params.role_id,
        )
        .await?;

        let role_role_grant = result
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(role_role_grant)
    }
}

pub async fn select_role_permission_grants_by_role_id_query(
    conn: &mut PgConnection,
    role_id: Uuid,
) -> Result<Vec<PgRolePermission>, BoxedError> {
    let result = sqlx::query_as::<_, PgRolePermission>(include_str!(
        "./select_role_permission_grants_by_role_id.sql"
    ))
    .bind(role_id)
    .fetch_all(conn)
    .await?;

    Ok(result)
}
