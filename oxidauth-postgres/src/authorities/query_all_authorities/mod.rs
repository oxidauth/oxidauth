use oxidauth_repository::authorities::query_all_authorities::*;

use crate::prelude::*;

#[async_trait]
impl QueryAllAuthorities for Database {
    async fn query_all_authorities(&self) -> Result<Vec<AuthorityRow>, QueryAllAuthoritiesError> {
        let result =
            sqlx::query_as::<_, super::PgAuthority>(include_str!("./query_all_authorities.sql"))
                .fetch_all(&self.pool)
                .await
                .map_err(|_| QueryAllAuthoritiesError {})?
                .into_iter()
                .map(Into::into)
                .collect();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::authorities::insert_authority::*;
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_query_all_authorities_successfully(pool: PgPool) {
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

        // @GEORGE - need a pattern for testing vec responses
        match db.query_all_authorities().await {
            Ok(authority) => {
                // assert_eq!(authority_id, authority.id);
                // assert_eq!(insert_params.name, authority.name);
            }
            Err(_) => unreachable!(),
        }
    }
}
