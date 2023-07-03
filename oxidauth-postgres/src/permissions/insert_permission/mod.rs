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
                .bind(&params.id)
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

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_insert_a_permission_by_id_successfully(pool: PgPool) {
        let db = Database { pool };

        let permission_id = Uuid::new_v4();

        let insert_params = InsertPermissionParams {
            id: Some(permission_id),
            realm: "Test".to_string(),
            resource: "Test".to_string(),
            action: "Test".to_string(),
        };

        match db.insert_permission(&insert_params).await {
            Ok(permission) => {
                assert_eq!(permission_id, permission.id);
                assert_eq!(insert_params.realm, permission.realm);
                assert_eq!(insert_params.resource, permission.resource);
                assert_eq!(insert_params.action, permission.action);
            }
            Err(_) => unreachable!(),
        }
    }
}
