use oxidauth_repository::permissions::delete_permission_by_id::*;

use crate::prelude::*;

use super::PgPermission;

#[async_trait]
impl DeletePermissionById for Database {
    async fn delete_permission_by_id(
        &self,
        permission_id: Uuid,
    ) -> Result<Permission, DeletePermissionByIdError> {
        let result = sqlx::query_as::<_, PgPermission>(include_str!(
            "./delete_permission_by_id.sql"
        ))
        .bind(permission_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| DeletePermissionByIdError {})?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::permissions::insert_permission::*;
    use sqlx::PgPool;

    use super::*;

    // #[sqlx::test]
    // async fn it_should_delete_a_permission_by_id_successfully(pool: PgPool) {
    //     let db = Database { pool };
    //
    //     let permission_id = Uuid::new_v4();
    //
    //     let insert_params = InsertPermissionParams {
    //         id: Some(permission_id),
    //         realm: "Test".to_string(),
    //         resource: "Test".to_string(),
    //         action: "Test".to_string(),
    //     };
    //
    //     db.insert_permission(&insert_params)
    //         .await
    //         .expect("should be able to insert permission");
    //
    //     match db
    //         .delete_permission_by_id(permission_id)
    //         .await
    //     {
    //         Ok(permission) => {
    //             assert_eq!(permission_id, permission.id);
    //             assert_eq!(
    //                 insert_params.realm,
    //                 permission.realm
    //             );
    //             assert_eq!(
    //                 insert_params.resource,
    //                 permission.resource
    //             );
    //             assert_eq!(
    //                 insert_params.action,
    //                 permission.action
    //             );
    //         },
    //         Err(_) => unreachable!(),
    //     }
    // }
}
