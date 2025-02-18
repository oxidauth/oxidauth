pub use oxidauth_http::{
    response::Response,
    server::api::v1::auth::register::{
        AuthorityStrategy, RegisterReq, RegisterRes,
    },
};
use oxidauth_kernel::{
    auth::oauth2::redirect::{Oauth2RedirectParams, Oauth2RedirectResponse},
    error::BoxedError,
};

pub use oxidauth_usecases::auth::strategies::*;

use super::*;

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn oauth2_redirect<T>(
        &self,
        params: T,
    ) -> Result<Response<Oauth2RedirectResponse>, BoxedError>
    where
        T: Into<Oauth2RedirectParams> + fmt::Debug,
    {
        let params = params.into();

        let result: Response<Oauth2RedirectResponse> = self
            .post(
                "/auth/oauth2/redirect",
                params,
            )
            .await?;

        Ok(result)
    }
}
