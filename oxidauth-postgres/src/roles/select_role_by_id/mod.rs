use oxidauth_kernel::roles::find_role_by_id::FindRoleById;
use oxidauth_repository::roles::select_role_by_id::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a FindRoleById> for Database {
    type Response = Role;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_role_by_id_query", skip(self))]
    async fn call(
        &self,
        params: &'a FindRoleById,
    ) -> Result<Role, BoxedError> {
        let result = sqlx::query_as::<_, RoleRow>(include_str!("./select_role_by_id.sql"))
            .bind(&params.role_id)
            .fetch_one(&self.pool)
            .await?;

        let role = result.into();

        Ok(role)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_a_role_by_id_successfully(pool: PgPool) {}
}
