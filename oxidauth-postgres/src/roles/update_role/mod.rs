use oxidauth_kernel::roles::update_role::UpdateRole;
use oxidauth_repository::roles::update_role::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a UpdateRole> for Database {
    type Response = Role;
    type Error = BoxedError;

    #[tracing::instrument(name = "update_role_query", skip(self))]
    async fn call(&self, params: &'a UpdateRole) -> Result<Role, BoxedError> {
        let result = sqlx::query_as::<_, PgRole>(include_str!(
            "./update_role.sql"
        ))
        .bind(params.role_id)
        .bind(&params.name)
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
    async fn it_should_update_a_role_successfully(_pool: PgPool) {}
}
