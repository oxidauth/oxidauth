use oxidauth_repository::permissions::query_permission_by_parts::*;

use crate::prelude::*;

#[async_trait]
impl QueryPermissionByParts for Database {
    async fn query_permission_by_parts(
        &self,
        params: &QueryPermissionByPartsParams,
    ) -> Result<PermissionRow, QueryPermissionByPartsError> {
        let result = sqlx::query_as::<_, super::PermissionRow>(include_str!(
            "./query_permission_by_parts.sql"
        ))
        .bind(&params.realm)
        .bind(&params.resource)
        .bind(&params.action)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| QueryPermissionByPartsError {})?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::permissions::insert_permission::*;
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_query_a_permission_by_parts_successfully(pool: PgPool) {
        let db = Database { pool };

        let permission_id = Uuid::new_v4();

        let insert_params = InsertPermissionParams {
            id: Some(permission_id),
            realm: "TestRealm".to_string(),
            resource: "TestResource".to_string(),
            action: "TestAction".to_string(),
        };

        let query_params = QueryPermissionByPartsParams {
            realm: "TestRealm".to_string(),
            resource: "TestResource".to_string(),
            action: "TestAction".to_string(),
        };

        db.insert_permission(&insert_params)
            .await
            .expect("should be able to insert permission");

        match db.query_permission_by_parts(&query_params).await {
            Ok(permission) => {
                assert_eq!(permission_id, permission.id);
            }
            Err(_) => unreachable!(),
        }
    }
}
