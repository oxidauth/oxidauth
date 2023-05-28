use crate::{
    prelude::*,
    rows::authority::{AuthorityRow, UpdateAuthority},
};

impl Database {
    async fn update_authority(&self, authority: UpdateAuthority) -> Result<AuthorityRow> {
        let mut conn = self.pool.acquire().await?;

        update_authority_query(&mut conn, authority).await
    }
}

pub async fn update_authority_query(
    conn: &mut PgConnection,
    authority: UpdateAuthority,
) -> Result<AuthorityRow> {
    let row = sqlx::query_as::<_, AuthorityRow>(include_str!("./update_authority.sql"))
        .bind(authority.id)
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
