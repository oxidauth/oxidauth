use crate::{
    prelude::*,
    rows::grants::{UserRoleGrant, UserRoleGrantRow},
};

impl Database {
    async fn delete_user_role_grant(&self, params: UserRoleGrant) -> Result<UserRoleGrantRow> {
        let mut conn = self.pool.acquire().await?;

        delete_user_role_grant_query(&mut conn, params).await
    }
}

pub async fn delete_user_role_grant_query(
    conn: &mut PgConnection,
    params: UserRoleGrant,
) -> Result<UserRoleGrantRow> {
    let row = sqlx::query_as::<_, UserRoleGrantRow>(include_str!("./delete_user_role_grant.sql"))
        .bind(params.user_id)
        .bind(params.role_id)
        .fetch_one(conn)
        .await?;

    Ok(row)
}
