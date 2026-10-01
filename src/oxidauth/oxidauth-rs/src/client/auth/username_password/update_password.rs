pub use oxidauth_http::Response;
pub use oxidauth_kernel::auth::username_password::update_password::{
    UpdatePasswordParams,
    UpdatePasswordResponse,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn username_password_update_password<T>(
        &self,
        params: T,
    ) -> Result<Response<UpdatePasswordResponse>, BoxedError>
    where
        T: Into<UpdatePasswordParams> + fmt::Debug,
    {
        let params = params.into();

        let result: Response<UpdatePasswordResponse> = self
            .post("/auth/username_password/update_password", params)
            .await?;

        Ok(result)
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::contract_raw;

    #[tokio::test]
    async fn username_password_update_password_route_contract() {
        contract_raw(
            "POST",
            "/api/v1/auth/username_password/update_password",
            json!({ "success": true }),
            |client| {
                async move {
                    client
                        .username_password_update_password(UpdatePasswordParams {
                            code: "CODE-123".to_string(),
                            username: "malreynolds".to_string(),
                            client_key: Uuid::new_v4(),
                            password: "new-pass".to_string(),
                            password_conf: "new-pass".to_string(),
                        })
                        .await
                }
            },
        )
        .await;
    }
}
