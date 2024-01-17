use oxidauth_kernel::roles::create_role::CreateRole;
use oxidauth_repository::roles::insert_role::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a CreateRole> for Database {
    type Response = Role;
    type Error = BoxedError;

    #[tracing::instrument(name = "insert_role_query", skip(self))]
    async fn call(
        &self,
        params: &'a CreateRole,
    ) -> Result<Role, BoxedError> {
        let result = sqlx::query_as::<_, PgRole>(include_str!("./insert_role.sql"))
            .bind(None::<Uuid>)
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

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_insert_a_role_successfully(pool: PgPool) {}
}
