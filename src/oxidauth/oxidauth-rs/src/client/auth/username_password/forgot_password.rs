pub use oxidauth_http::Response;
pub use oxidauth_kernel::auth::username_password::forgot_password::{
    ForgotPasswordParams,
    ForgotPasswordResponse,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn username_password_forgot_password<T>(
        &self,
        params: T,
    ) -> Result<Response<ForgotPasswordResponse>, BoxedError>
    where
        T: Into<ForgotPasswordParams> + fmt::Debug,
    {
        let params = params.into();

        let result: Response<ForgotPasswordResponse> = self
            .post("/auth/username_password/forgot_password", params)
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
    async fn username_password_forgot_password_route_contract() {
        let user_id = Uuid::new_v4();

        contract_raw(
            "POST",
            "/api/v1/auth/username_password/forgot_password",
            json!({ "code": "CODE-123" }),
            move |client| {
                async move {
                    client
                        .username_password_forgot_password(ForgotPasswordParams { user_id })
                        .await
                }
            },
        )
        .await;
    }
}
