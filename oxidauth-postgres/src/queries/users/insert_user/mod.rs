use crate::prelude::*;

impl Database {
    pub async fn insert_user(&self, user: impl Into<UserCreateRow>) -> Result<UserRow> {
        let mut conn = self.pool.acquire().await?;

        insert_user_query(&mut conn, user).await
    }
}

pub async fn insert_user_query(
    conn: &mut PgConnection,
    user: impl Into<UserCreateRow>,
) -> Result<UserRow> {
    let user = user.into();

    let row = sqlx::query_as::<_, UserRow>(include_str!("./insert_user.sql"))
        .bind(user.username)
        .bind(user.email)
        .bind(user.first_name)
        .bind(user.last_name)
        .bind(user.status)
        .bind(user.kind)
        .bind(user.profile)
        .fetch_one(conn)
        .await?;

    Ok(row)
}
