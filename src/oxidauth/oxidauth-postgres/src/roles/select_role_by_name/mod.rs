use oxidauth_kernel::roles::find_role_by_name::FindRoleByName;
use oxidauth_repository::roles::select_role_by_name::*;
use sqlx::PgConnection;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a FindRoleByName> for Database {
    type Response = Role;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_role_by_name_query", skip(self))]
    async fn call(
        &self,
        params: &'a FindRoleByName,
    ) -> Result<Role, BoxedError> {
        let pool = self.read_pool();
        let mut conn = pool.acquire().await?;

        let result = select_role_by_name_query(&mut conn, &params.role).await?;

        let role = result.into();

        Ok(role)
    }
}

pub async fn select_role_by_name_query(
    conn: &mut PgConnection,
    role: &String,
) -> Result<PgRole, BoxedError> {
    let result = sqlx::query_as::<_, PgRole>(include_str!(
        "./select_role_by_name.sql"
    ))
    .bind(role)
    .fetch_one(conn)
    .await?;

    Ok(result)
}



