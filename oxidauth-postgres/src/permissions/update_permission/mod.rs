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
mod tests {
    use oxidauth_repository::permissions::insert_permission::*;
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_udpate_a_permission_successfully(pool: PgPool) {
        let db = Database { pool };

        let permission_id = Uuid::new_v4();

        let insert_params = InsertPermissionParams {
            id: Some(permission_id),
            realm: "Test".to_string(),
            resource: "Test".to_string(),
            action: "Test".to_string(),
        };

        let update_params = UpdatePermissionParams {
            id: permission_id,
            realm: "UpdatedRealm".to_string(),
            resource: "UpdatedResource".to_string(),
            action: "UpdatedAction".to_string(),
        };

        db.insert_permission(&insert_params)
            .await
            .expect("should be able to insert permission");

        match db.update_permission(&update_params).await {
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
