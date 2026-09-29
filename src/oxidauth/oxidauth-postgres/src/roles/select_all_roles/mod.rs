use oxidauth_kernel::roles::list_all_roles::ListAllRoles;
use oxidauth_repository::roles::select_all_roles::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a ListAllRoles> for Database {
    type Response = Vec<Role>;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_all_roles_query", skip(self))]
    async fn call(
        &self,
        _params: &'a ListAllRoles,
    ) -> Result<Vec<Role>, BoxedError> {
        let result = sqlx::query_as::<_, PgRole>(include_str!(
            "./select_all_roles.sql"
        ))
        .fetch_all(&self.read_pool())
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

        Ok(result)
    }
}



