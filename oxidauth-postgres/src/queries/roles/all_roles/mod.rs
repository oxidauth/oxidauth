use crate::prelude::*;

impl Database {
    async fn all_roles(&self) -> Result<Vec<RoleRow>> {
        let mut conn = self.pool.acquire().await?;

        all_roles_query(&mut conn).await
    }
}

pub async fn all_roles_query(conn: &mut PgConnection) -> Result<Vec<RoleRow>> {
    let rows = sqlx::query_as::<_, RoleRow>(include_str!("./all_roles.sql"))
        .fetch_all(conn)
        .await?;

    Ok(rows)
}
