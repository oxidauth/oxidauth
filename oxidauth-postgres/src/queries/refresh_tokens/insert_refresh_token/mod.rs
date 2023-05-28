use crate::{
    prelude::*,
    rows::refresh_token::{InsertRefreshToken, RefreshTokenRow},
};

impl Database {
    async fn insert_refresh_token(&self, params: InsertRefreshToken) -> Result<RefreshTokenRow> {
        let mut conn = self.pool.acquire().await?;

        insert_refresh_token_query(&mut conn, params).await
    }
}

pub async fn insert_refresh_token_query(
    conn: &mut PgConnection,
    params: InsertRefreshToken,
) -> Result<RefreshTokenRow> {
    let row = sqlx::query_as::<_, RefreshTokenRow>(include_str!("./insert_refresh_token.sql"))
        .bind(params.user_id)
        .bind(params.authority_id)
        .bind(params.expires_at)
        .fetch_one(conn)
        .await?;

    Ok(row)
}
