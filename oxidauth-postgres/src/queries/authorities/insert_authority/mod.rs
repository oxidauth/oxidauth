use crate::{
    prelude::*,
    rows::authority::{AuthorityRow, InsertAuthority},
};

impl Database {
    async fn insert_authority(&self, authority: InsertAuthority) -> Result<AuthorityRow> {
        let mut conn = self.pool.acquire().await?;

        insert_authority_query(&mut conn, authority).await
    }
}

pub async fn insert_authority_query(
    conn: &mut PgConnection,
    authority: InsertAuthority,
) -> Result<AuthorityRow> {
    let row = sqlx::query_as::<_, AuthorityRow>(include_str!("./insert_authority.sql"))
        .bind(authority.name)
        .bind(authority.client_key)
        .bind(authority.status)
        .bind(authority.strategy)
        .bind(authority.params)
        .bind(authority.settings)
        .fetch_one(conn)
        .await?;

    Ok(row)
}
