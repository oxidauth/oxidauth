use crate::prelude::*;

impl Database {
    async fn find_role_by_name(&self) -> Result<RoleRow> {
        let mut conn = self.pool.acquire().await?;

        find_role_by_name_query(&mut conn).await
    }
}

pub async fn find_role_by_name_query(conn: &mut PgConnection) -> Result<RoleRow> {
    let row = sqlx::query_as::<_, RoleRow>(include_str!("./find_role_by_name.sql"))
        .fetch_one(conn)
        .await?;

    Ok(row)
}
