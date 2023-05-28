use crate::{
    prelude::*,
    rows::grants::{UserPermissionGrant, UserPermissionGrantRow},
};

impl Database {
    async fn insert_user_permission_grant(
        &self,
        params: UserPermissionGrant,
    ) -> Result<UserPermissionGrantRow> {
        let mut conn = self.pool.acquire().await?;

        insert_user_permission_grant_query(&mut conn, params).await
    }
}

pub async fn insert_user_permission_grant_query(
    conn: &mut PgConnection,
    params: UserPermissionGrant,
) -> Result<UserPermissionGrantRow> {
    let row = sqlx::query_as::<_, UserPermissionGrantRow>(include_str!(
        "./insert_user_permission_grant.sql"
    ))
    .bind(params.user_id)
    .bind(params.permission_id)
    .fetch_one(conn)
    .await?;

    Ok(row)
}
