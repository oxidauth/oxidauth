use oxidauth_kernel::user_role_grants::UserRoleGrant;
use oxidauth_repository::user_role_grants::delete_user_role_grant::*;

use crate::prelude::*;

use super::PgUserRoleGrant;

#[async_trait]
impl DeleteUserRoleGrant for Database {
    async fn delete_user_role_grant(
        &self,
        params: &DeleteUserRoleGrantParams,
    ) -> Result<UserRoleGrant, DeleteUserRoleGrantError> {
        let result = sqlx::query_as::<_, PgUserRoleGrant>(include_str!(
            "./delete_user_role_grant.sql"
        ))
        .bind(params.user_id)
        .bind(params.role_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| DeleteUserRoleGrantError {})?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::user_role_grants::insert_user_role_grant::*;
    use sqlx::PgPool;

    use super::*;

    // #[sqlx::test]
    // async fn it_should_delete_a_user_role_grant_successfully(pool: PgPool) {
    //     let db = Database { pool };
    //
    //     let user_id = Uuid::new_v4();
    //     let role_id = Uuid::new_v4();
    //
    //     let insert_params = InsertUserRoleGrantParams { user_id, role_id };
    //
    //     let delete_params = DeleteUserRoleGrantParams { user_id, role_id };
    //
    //     db.insert_user_role_grant(&insert_params)
    //         .await
    //         .expect("should be able to insert user role grant");
    //
    //     match db
    //         .delete_user_role_grant(&delete_params)
    //         .await
    //     {
    //         Ok(user_role_grant) => {
    //             assert_eq!(
    //                 user_id,
    //                 user_role_grant.user_id
    //             );
    //             assert_eq!(
    //                 role_id,
    //                 user_role_grant.role_id
    //             );
    //         },
    //         Err(_) => unreachable!(),
    //     }
    // }
}
