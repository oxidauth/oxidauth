use oxidauth_repository::public_keys::delete_public_key_by_id::*;

use crate::prelude::*;

#[async_trait]
impl DeletePublicKeyById for Database {
    async fn delete_public_key_by_id(
        &self,
        public_key_id: Uuid,
    ) -> Result<PublicKeyRow, DeletePublicKeyByIdError> {
        let result =
            sqlx::query_as::<_, super::PublicKeyRow>(include_str!("./delete_public_key_by_id.sql"))
                .bind(public_key_id)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| DeletePublicKeyByIdError {})?;

        Ok(result)
    }
}
#[cfg(test)]
mod tests {
    use oxidauth_repository::public_keys::insert_public_key::*;
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_delete_a_public_key_by_id_successfully(pool: PgPool) {
        let db = Database { pool };

        let public_key_id = Uuid::new_v4();

        let insert_params = InsertPublicKeyParams {
            id: Some(public_key_id),
            public_key: [1, 1, 1, 1].to_vec(),
            private_key: [2, 2, 2, 2].to_vec(),
        };

        db.insert_public_key(&insert_params)
            .await
            .expect("should be able to insert public_key");

        match db.delete_public_key_by_id(public_key_id).await {
            Ok(public_key) => {
                assert_eq!(public_key_id, public_key.id);
                assert_eq!(insert_params.public_key, public_key.public_key);
            }
            Err(_) => unreachable!(),
        }
    }
}
