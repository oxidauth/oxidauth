use axum::{Json, extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tracing::info;

use oxidauth_kernel::{auth::oauth2::redirect::*, error::IntoOxidAuthError};

use crate::{provider::Provider, response::Response};

pub type Oauth2RedirectReq = Oauth2RedirectParams;

#[derive(Debug, Serialize, Deserialize)]
pub struct Oauth2RedirectRes {
    pub redirect_url: String,
}

#[tracing::instrument(name = "oauth2_redirect_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<Oauth2RedirectParams>,
) -> impl IntoResponse {
    let service = provider.fetch::<Oauth2RedirectService>();

    let result = service.call(&params).await;

    match result {
        Ok(res) => {
            info!(
                message = "successfully return oauth2 redirect url",
                response = ?res,
            );

            Response::success().payload(Oauth2RedirectRes {
                redirect_url: res.redirect_url.to_string(),
            })
        },
        Err(err) => {
            info!(
                message = "failed to return oauth2 redirect url",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}
