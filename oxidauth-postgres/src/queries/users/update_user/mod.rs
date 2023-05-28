use crate::prelude::*;

impl Database {
    pub async fn update_user(&self, user: impl Into<UserUpdateRow>) -> Result<UserRow> {
        let mut conn = self.pool.acquire().await?;

        update_user_query(&mut conn, user).await
    }
}

pub async fn update_user_query(
    conn: &mut PgConnection,
    user: impl Into<UserUpdateRow>,
) -> Result<UserRow> {
    let user = user.into();

    let row = sqlx::query_as::<_, UserRow>(include_str!("./update_user.sql"))
        .bind(user.email)
        .bind(user.first_name)
        .bind(user.last_name)
        .bind(user.status)
        .bind(user.profile)
        .fetch_one(conn)
        .await?;

    Ok(row)
}
