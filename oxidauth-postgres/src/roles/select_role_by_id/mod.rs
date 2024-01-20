use oxidauth_kernel::roles::find_role_by_id::FindRoleById;
use oxidauth_repository::roles::select_role_by_id::*;
use sqlx::PgConnection;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a FindRoleById> for Database {
    type Response = Role;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_role_by_id_query", skip(self))]
    async fn call(&self, params: &'a FindRoleById) -> Result<Role, BoxedError> {
        let mut conn = self.pool.acquire().await?;

        let result = select_role_by_id_query(&mut conn, params.role_id).await?;

        let role = result.into();

        Ok(role)
    }
}

pub async fn select_role_by_id_query(
    conn: &mut PgConnection,
    role_id: Uuid,
) -> Result<PgRole, BoxedError> {
    let result = sqlx::query_as::<_, PgRole>(include_str!(
        "./select_role_by_id.sql"
    ))
    .bind(role_id)
    .fetch_one(conn)
    .await?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_a_role_by_id_successfully(pool: PgPool) {}
}
