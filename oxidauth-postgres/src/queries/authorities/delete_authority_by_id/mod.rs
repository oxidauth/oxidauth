use crate::{prelude::*, rows::authority::AuthorityRow};

impl Database {
    async fn delete_authority_by_id(&self, authority_id: Uuid) -> Result<AuthorityRow> {
        let mut conn = self.pool.acquire().await?;

        delete_authority_by_id_query(&mut conn, authority_id).await
    }
}

pub async fn delete_authority_by_id_query(
    conn: &mut PgConnection,
    authority_id: Uuid,
) -> Result<AuthorityRow> {
    let row = sqlx::query_as::<_, AuthorityRow>(include_str!("./delete_authority_by_id.sql"))
        .bind(authority_id)
        .fetch_one(conn)
        .await?;

    Ok(row)
}
