use crate::{prelude::*, rows::refresh_token::RefreshTokenRow};

impl Database {
    async fn find_refresh_token_by_id(&self, refresh_token_id: Uuid) -> Result<RefreshTokenRow> {
        let mut conn = self.pool.acquire().await?;

        find_refresh_token_by_id_query(&mut conn, refresh_token_id).await
    }
}

pub async fn find_refresh_token_by_id_query(
    conn: &mut PgConnection,
    refresh_token_id: Uuid,
) -> Result<RefreshTokenRow> {
    let row = sqlx::query_as::<_, RefreshTokenRow>(include_str!("./find_refresh_token_by_id.sql"))
        .bind(refresh_token_id)
        .fetch_one(conn)
        .await?;

    Ok(row)
}
