use oxidauth_repository::user_authorities::delete_user_authority_by_id::*;

use crate::prelude::*;

#[async_trait]
impl DeleteUserAuthorityById for Database {
    async fn delete_user_authority_by_id(
        &self,
        user_authority_id: Uuid,
    ) -> Result<UserAuthorityRow, DeleteUserAuthorityByIdError> {
        let result = sqlx::query_as::<_, super::UserAuthorityRow>(include_str!(
            "./delete_user_authority_by_id.sql"
        ))
        .bind(user_authority_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| DeleteUserAuthorityByIdError {})?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::user_authorities::insert_user_authority::*;
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_delete_a_user_authority_successfully(pool: PgPool) {
        // let db = Database { pool };
        //
        // let authority_id = Uuid::new_v4();
        //
        //
        // };
        //
        // db.insert_authority(&insert_params)
        //     .await
        //     .expect("should be able to insert authority");
        //
        // match db.delete_authority_by_id(authority_id).await {
        //     Ok(authority) => {
        //         assert_eq!(authority_id, authority.id);
        //         assert_eq!(insert_params.name, authority.name);
        //     }
        //     Err(_) => unreachable!(),
        // }
    }
}
