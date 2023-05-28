use crate::{
    prelude::*,
    rows::public_key::{InsertPublicKey, PublicKeyRow},
};

impl Database {
    async fn insert_public_key(&self, params: InsertPublicKey) -> Result<PublicKeyRow> {
        let mut conn = self.pool.acquire().await?;

        insert_public_key_query(&mut conn, params).await
    }
}

pub async fn insert_public_key_query(
    conn: &mut PgConnection,
    params: InsertPublicKey,
) -> Result<PublicKeyRow> {
    let row = sqlx::query_as::<_, PublicKeyRow>(include_str!("./insert_public_key.sql"))
        .bind(params.public_key)
        .bind(params.private_key)
        .fetch_one(conn)
        .await?;

    Ok(row)
}
