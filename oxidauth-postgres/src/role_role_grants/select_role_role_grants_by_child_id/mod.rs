use crate::prelude::*;
use sqlx::PgConnection;

use super::PgRoleRoleGrantDetail;

pub async fn select_role_role_grants_by_child_id_query(
    conn: &mut PgConnection,
    role_id: Uuid,
) -> Result<Vec<PgRoleRoleGrantDetail>, BoxedError> {
    let result = sqlx::query_as::<_, PgRoleRoleGrantDetail>(include_str!(
        "./select_role_role_grants_by_child_id.sql"
    ))
    .bind(role_id)
    .fetch_all(conn)
    .await?;

    Ok(result)
}
