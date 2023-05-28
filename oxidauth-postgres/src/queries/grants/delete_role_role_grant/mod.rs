use crate::{
    prelude::*,
    rows::grants::{RoleRoleGrant, RoleRoleGrantRow},
};

impl Database {
    async fn delete_role_role_grant(&self, params: RoleRoleGrant) -> Result<RoleRoleGrantRow> {
        let mut conn = self.pool.acquire().await?;

        delete_role_role_grant_query(&mut conn, params).await
    }
}

pub async fn delete_role_role_grant_query(
    conn: &mut PgConnection,
    params: RoleRoleGrant,
) -> Result<RoleRoleGrantRow> {
    let row = sqlx::query_as::<_, RoleRoleGrantRow>(include_str!("./delete_role_role_grant.sql"))
        .bind(params.parent_id)
        .bind(params.child_id)
        .fetch_one(conn)
        .await?;

    Ok(row)
}
