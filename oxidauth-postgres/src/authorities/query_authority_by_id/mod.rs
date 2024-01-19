use oxidauth_repository::authorities::query_authority_by_id::*;

use crate::prelude::*;

#[async_trait]
impl QueryAuthorityById for Database {
    async fn query_authority_by_id(
        &self,
        authority_id: Uuid,
    ) -> Result<AuthorityRow, QueryAuthorityByIdError> {
        let result =
            sqlx::query_as::<_, super::PgAuthority>(include_str!("./query_authority_by_id.sql"))
                .bind(authority_id)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| QueryAuthorityByIdError {})?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::authorities::insert_authority::*;
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_query_an_authority_by_id_successfully(pool: PgPool) {
        let db = Database { pool };

        let authority_id = Uuid::new_v4();

        let insert_params = InsertAuthorityParams {
            id: Some(authority_id),
            name: "Test".to_string(),
            client_key: Uuid::new_v4(),
            status: "Test".to_string(),
            strategy: "Test".to_string(),
            settings: serde_json::Value::default(),
            params: serde_json::Value::default(),
        };

        db.insert_authority(&insert_params)
            .await
            .expect("should be able to insert authority");

        match db.query_authority_by_id(authority_id).await {
            Ok(authority) => {
                assert_eq!(authority_id, authority.id);
                assert_eq!(insert_params.name, authority.name);
            }
            Err(_) => unreachable!(),
        }
    }
}
