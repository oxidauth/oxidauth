use crate::{prelude::*, rows::authority::AuthorityRow};

impl Database {
    async fn find_authority_by_id(&self, authority_id: Uuid) -> Result<AuthorityRow> {
        let mut conn = self.pool.acquire().await?;

        find_role_permission_grants_by_role_id_query(&mut conn, authority_id).await
    }
}

pub async fn find_role_permission_grants_by_role_id_query(
    conn: &mut PgConnection,
    authority_id: Uuid,
) -> Result<AuthorityRow> {
    let row = sqlx::query_as::<_, AuthorityRow>(include_str!("./find_authority_by_id.sql"))
        .bind(authority_id)
        .fetch_one(conn)
        .await?;

    Ok(row)
}
