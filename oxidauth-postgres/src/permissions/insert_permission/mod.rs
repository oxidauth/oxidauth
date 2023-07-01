use oxidauth_repository::permissions::insert_permission::*;

use crate::prelude::*;

#[async_trait]
impl InsertPermission for Database {
    async fn insert_permission(
        &self,
        params: &InsertPermissionParams,
    ) -> Result<PermissionRow, InsertPermissionError> {
        let result =
            sqlx::query_as::<_, super::PermissionRow>(include_str!("insert_permission.sql"))
                .bind(&params.realm)
                .bind(&params.resource)
                .bind(&params.action)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| InsertPermissionError {})?;

        Ok(result)
    }
}
