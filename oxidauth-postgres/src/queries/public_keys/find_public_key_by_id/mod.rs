use crate::{prelude::*, rows::public_key::PublicKeyRow};

impl Database {
    async fn find_public_key_by_id(&self, public_key_id: Uuid) -> Result<PublicKeyRow> {
        let mut conn = self.pool.acquire().await?;

        find_public_key_by_id_query(&mut conn, public_key_id).await
    }
}

pub async fn find_public_key_by_id_query(
    conn: &mut PgConnection,
    public_key_id: Uuid,
) -> Result<PublicKeyRow> {
    let row = sqlx::query_as::<_, PublicKeyRow>(include_str!("./find_public_key_by_id.sql"))
        .bind(public_key_id)
        .fetch_one(conn)
        .await?;

    Ok(row)
}
