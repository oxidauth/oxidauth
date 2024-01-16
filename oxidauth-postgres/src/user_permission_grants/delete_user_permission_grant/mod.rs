use oxidauth_kernel::user_permission_grants::UserPermissionGrant;
use oxidauth_repository::user_permission_grants::delete_user_permission_grant::*;

use crate::prelude::*;

use super::PgUserPermissionGrant;

#[async_trait]
impl DeleteUserPermissionGrant for Database {
    async fn delete_user_permission_grant(
        &self,
        params: &DeleteUserPermissionGrantParams,
    ) -> Result<UserPermissionGrant, DeleteUserPermissionGrantError> {
        let result = sqlx::query_as::<_, super::PgUserPermissionGrant>(
            include_str!("./delete_user_permission_grant.sql"),
        )
        .bind(params.user_id)
        .bind(params.permission_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| DeleteUserPermissionGrantError {})?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::user_permission_grants::insert_user_permission_grant::*;
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_delete_a_user_permission_grant_successfully(
        pool: PgPool,
    ) {
        // let db = Database { pool };
        //
        // let user_id = Uuid::new_v4();
        // let permission_id = Uuid::new_v4();
        //
        // let insert_params = InsertUserRoleGrantParams {
        //     user_id: user_id,
        //     permission_id: permission_id,
        // };
        //
        // let delete_params = DeleteUserRoleGrantParams {
        //     user_id: user_id,
        //     permission_id: permission_id,
        // };
        //
        // db.insert_user_permission_grant(&insert_params)
        //     .await
        //     .expect("should be able to insert user permission grant");
        //
        // match db.delete_user_permission_grant(&delete_params).await {
        //     Ok(user_permission_grant) => {
        //         assert_eq!(user_id, user_permission_grant.user_id);
        //         assert_eq!(permission_id, user_permission_grant.permission_id);
        //     }
        //     Err(_) => unreachable!(),
        // }
    }
}
