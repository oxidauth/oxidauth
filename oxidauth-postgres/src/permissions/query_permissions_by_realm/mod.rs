use oxidauth_repository::permissions::query_permissions_by_realm::*;

use crate::prelude::*;

#[async_trait]
impl QueryPermissionsByRealm for Database {
    async fn query_permissions_by_realm(
        &self,
        realm: String,
    ) -> Result<PermissionRow, QueryPermissionsByRealmError> {
        let result = sqlx::query_as::<_, super::PermissionRow>(include_str!(
            "./query_permissions_by_realm.sql"
        ))
        .bind(realm)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| QueryPermissionsByRealmError {})?
        .into_iter()
        .map(Into::into)
        .collect();

        Ok(result)
    }
}
