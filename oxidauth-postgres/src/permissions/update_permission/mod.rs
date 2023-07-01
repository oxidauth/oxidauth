use oxidauth_repository::permissions::update_permission::*;

use crate::prelude::*;

#[async_trait]
impl UpdatePermission for Database {
    async fn update_permission(
        &self,
        params: &UpdatePermissionParams,
    ) -> Result<PermissionRow, UpdatePermissionError> {
        let result =
            sqlx::query_as::<_, super::PermissionRow>(include_str!("./update_permission.sql"))
                .bind(&params.id)
                .bind(&params.realm)
                .bind(&params.resource)
                .bind(&params.action)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| UpdatePermissionError {})?;

        Ok(result)
    }
}
