use oxidauth_kernel::private_keys::PrivateKey;
use rand::rngs::StdRng;
use uuid::Uuid;

#[async_trait]
impl<'a> Service<&'a InsertAuthKeyParams> for Database {
    type Response = AuthKey;
    type Error = BoxedError;

    #[tracing::instrument(name = "insert_auth_key_query", skip(self, params))]
    async fn call(
        &self,
        params: &'a InsertAuthKeyParams,
    ) -> Result<Self::Response, Self::Error> {
        // generate random vec
        let mut nums: Vec<i32> = (1..100).collect();
        let mut rng = rand::thread_rng();
        nums.shuffle(&mut rng);

        let result = sqlx::query_as::<_, PgAuthKey>(include_str!(
            "./insert_auth_key.sql"
        ))
        .bind(&params.user_id)
        .bind(nums)
        .fetch_one(&self.pool)
        .await?;

        let public_key = result.try_into()?;

        Ok(public_key)
    }
}
