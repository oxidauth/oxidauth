use oxidauth_kernel::roles::delete_role::DeleteRole;
use oxidauth_repository::roles::delete_role::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a DeleteRole> for Database {
    type Response = Role;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_role_query", skip(self))]
    async fn call(&self, params: &'a DeleteRole) -> Result<Role, BoxedError> {
        let result = sqlx::query_as::<_, PgRole>(include_str!(
            "./delete_role.sql"
        ))
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

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_a_role_by_id_successfully(_pool: PgPool) {}
}
