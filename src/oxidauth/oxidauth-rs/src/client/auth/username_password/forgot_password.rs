pub use oxidauth_http::response::Response;
pub use oxidauth_kernel::auth::username_password::forgot_password::{
    ForgotPasswordParams, ForgotPasswordResponse,
};

use oxidauth_kernel::error::BoxedError;
pub use oxidauth_usecases::auth::strategies::*;

use super::*;

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn username_password_forgot_password<T>(
        &self,
        params: ForgotPasswordParams,
    ) -> Result<Response<ForgotPasswordResponse>, BoxedError>
    where
        T: Into<ForgotPasswordParams> + fmt::Debug,
    {
        let result: Response<ForgotPasswordResponse> = self
            .post("/auth/username_password/forgot_password", params)
            .await?;

        Ok(result)
    }
}
