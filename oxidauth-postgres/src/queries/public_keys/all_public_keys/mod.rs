use crate::{prelude::*, rows::public_key::PublicKeyRow};

impl Database {
    async fn all_public_keys(&self) -> Result<Vec<PublicKeyRow>> {
        let mut conn = self.pool.acquire().await?;

        all_public_keys_query(&mut conn).await
    }
}

pub async fn all_public_keys_query(conn: &mut PgConnection) -> Result<Vec<PublicKeyRow>> {
    let row = sqlx::query_as::<_, PublicKeyRow>(include_str!("./all_public_keys.sql"))
        .fetch_all(conn)
        .await?;

    Ok(row)
}
