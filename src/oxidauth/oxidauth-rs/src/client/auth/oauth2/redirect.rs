pub use oxidauth_http::{
    Response,
    auth::{
        oauth2::redirect::Oauth2RedirectRes,
        register::{RegisterReq, RegisterRes},
    },
};
pub use oxidauth_kernel::authorities::AuthorityStrategy;
use oxidauth_kernel::{auth::oauth2::redirect::Oauth2RedirectParams, error::BoxedError};
pub use oxidauth_services::auth::strategies::*;

use super::*;

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn oauth2_redirect<T>(
        &self,
        params: T,
    ) -> Result<Response<Oauth2RedirectRes>, BoxedError>
    where
        T: Into<Oauth2RedirectParams> + fmt::Debug,
    {
        let params = params.into();

        let result: Response<Oauth2RedirectRes> = self
            .post("/auth/oauth2/redirect", params)
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
    async fn oauth2_redirect_route_contract() {
        contract_raw(
            "POST",
            "/api/v1/auth/oauth2/redirect",
            json!({
                "redirect_url": "https://accounts.google.com/o/oauth2/v2/auth?client_id=oxidauth&prompt=consent"
            }),
            |client| async move {
                client
                    .oauth2_redirect(Oauth2RedirectParams {
                        client_key: Uuid::new_v4(),
                        email: None,
                    })
                    .await
            },
        )
        .await;
    }
}
