pub use oxidauth_http::response::Response;

pub use oxidauth_kernel::auth::username_password::update_password::{
    UpdatePasswordParams, UpdatePasswordResponse,
};
use oxidauth_kernel::error::BoxedError;
pub use oxidauth_usecases::auth::strategies::*;

use super::*;

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn username_password_update_password<T>(
        &self,
        params: UpdatePasswordParams,
    ) -> Result<Response<UpdatePasswordResponse>, BoxedError>
    where
        T: Into<UpdatePasswordParams> + fmt::Debug,
    {
        let result: Response<UpdatePasswordResponse> = self
            .post("/auth/username_password/update_password", params)
            .await?;

        Ok(result)
    }
}
