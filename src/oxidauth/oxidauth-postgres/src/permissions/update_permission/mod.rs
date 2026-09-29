use oxidauth_repository::permissions::update_permission::*;

use crate::prelude::*;

use super::PgPermission;

#[async_trait]
impl UpdatePermission for Database {
    async fn update_permission(
        &self,
        params: &UpdatePermissionParams,
    ) -> Result<Permission, UpdatePermissionError> {
        let result = sqlx::query_as::<_, PgPermission>(include_str!(
            "./update_permission.sql"
        ))
        .bind(params.id)
        .bind(&params.realm)
        .bind(&params.resource)
        .bind(&params.action)
        .fetch_one(&self.write_pool())
        .await
        .map(Into::into)
        .map_err(|_| UpdatePermissionError {})?;

        Ok(result)
    }
}

