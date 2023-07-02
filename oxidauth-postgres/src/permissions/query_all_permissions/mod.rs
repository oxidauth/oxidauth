use oxidauth_repository::permissions::query_all_permissions::*;

use crate::prelude::*;

#[async_trait]
impl QueryAllPermissions for Database {
    async fn query_all_permissions(&self) -> Result<Vec<PermissionRow>, QueryAllPermissionsError> {
        let result =
            sqlx::query_as::<_, super::PermissionRow>(include_str!("./query_all_permissions.sql"))
                .fetch_all(&self.pool)
                .await
                .map_err(|_| QueryAllPermissionsError {})?
                .into_iter()
                .map(Into::into)
                .collect();

        Ok(result)
    }
}
