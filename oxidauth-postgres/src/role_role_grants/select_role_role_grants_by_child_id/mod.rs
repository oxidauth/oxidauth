use oxidauth_kernel::{
    role_role_grants::{
        list_role_role_grants_by_child_id::*,
        RoleRoleGrant,
        RoleRoleGrantDetail,
    },
    roles::Role,
};
use oxidauth_repository::role_role_grants::select_role_role_grants_by_child_id::*;

use crate::prelude::*;

use super::PgRoleRoleGrantDetail;

#[async_trait]
impl<'a> Service<&'a ListRoleRoleGrantsByChildId> for Database {
    type Response = Vec<RoleRoleGrantDetail>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_role_role_grants_by_child_id_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a ListRoleRoleGrantsByChildId,
    ) -> Result<Vec<RoleRoleGrantDetail>, BoxedError> {
        let mut conn = self.pool.acquire().await?;

        let result = select_role_role_grants_by_child_id_query(
            &mut conn,
            params.child_id,
        )
        .await?;

        let role_role_grant = result
            .into_iter()
            .map(|pg| RoleRoleGrantDetail {
                role: Role {
                    id: pg.parent_id,
                    name: pg.role_name,
                    created_at: pg.role_created_at,
                    updated_at: pg.role_updated_at,
                },
                grant: RoleRoleGrant {
                    parent_id: pg.parent_id,
                    child_id: pg.child_id,
                    created_at: pg.created_at,
                    updated_at: pg.updated_at,
                },
            })
            .collect();

        Ok(role_role_grant)
    }
}

pub async fn select_role_role_grants_by_child_id_query(
    conn: &mut PgConnection,
    role_id: Uuid,
) -> Result<Vec<PgRoleRoleGrantDetail>, BoxedError> {
    let result = sqlx::query_as::<_, PgRoleRoleGrantDetail>(include_str!(
        "./select_role_role_grants_by_child_id.sql"
    ))
    .bind(role_id)
    .fetch_all(conn)
    .await?;

    Ok(result)
}
