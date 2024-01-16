use oxidauth_repository::permissions::query_permissions_by_realm::*;

use crate::prelude::*;

use super::PgPermission;

#[async_trait]
impl QueryPermissionsByRealm for Database {
    async fn query_permissions_by_realm(
        &self,
        realm: String,
    ) -> Result<Vec<Permission>, QueryPermissionsByRealmError> {
        let result = sqlx::query_as::<_, PgPermission>(include_str!(
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

#[cfg(test)]
mod tests {
    use oxidauth_repository::permissions::insert_permission::*;
    use sqlx::PgPool;

    use super::*;

    // #[sqlx::test]
    // async fn it_should_query_a_permissions_by_realm_successfully(pool: PgPool) {
    //     let db = Database { pool };
    //
    //     let permission_id = Uuid::new_v4();
    //
    //     let insert_params = InsertPermissionParams {
    //         id: Some(permission_id),
    //         realm: "TestRealm".to_string(),
    //         resource: "TestResource".to_string(),
    //         action: "TestAction".to_string(),
    //     };
    //
    //     db.insert_permission(&insert_params)
    //         .await
    //         .expect("should be able to insert permission");
    //
    //     match db.query_permissions_by_realm().await {
    //         Ok(permission) => {
    //             assert_eq!(permission_id, permission.id);
    //         }
    //         Err(_) => unreachable!(),
    //     }
    // }
}
