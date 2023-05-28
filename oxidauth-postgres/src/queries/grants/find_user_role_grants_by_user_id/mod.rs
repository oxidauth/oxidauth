use crate::{prelude::*, rows::grants::UserRoleGrantRow};

impl Database {
    async fn find_user_role_grants_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserRoleGrantRow>> {
        let mut conn = self.pool.acquire().await?;

        find_user_role_grants_by_user_id_query(&mut conn, user_id).await
    }
}

pub async fn find_user_role_grants_by_user_id_query(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<UserRoleGrantRow>> {
    let row = sqlx::query_as::<_, UserRoleGrantRow>(include_str!(
        "./find_user_role_grants_by_user_id.sql"
    ))
    .bind(user_id)
    .fetch_all(conn)
    .await?;

    Ok(row)
}
