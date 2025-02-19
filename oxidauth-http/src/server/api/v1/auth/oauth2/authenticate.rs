use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_kernel::auth::oauth2::authenticate::Oauth2Authenticate;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{provider::Provider, response::Response};
use oxidauth_kernel::{
    auth::oauth2::authenticate::*, error::IntoOxidAuthError,
};

pub type Oauth2AuthenticateReq = Oauth2AuthenticateParams;
pub type Oauth2AuthenticateRes = Oauth2AuthenticateResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct Oauth2AuthenticatePathParams {
    pub client_key: Uuid,
}

#[tracing::instrument(name = "oauth2 authenticate", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(path_params): Path<Oauth2AuthenticatePathParams>,
    Json(params): Json<Oauth2AuthenticateReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<Oauth2AuthenticateService>();

    let result = service
        .call(&Oauth2Authenticate {
            code: params.code,
            scope: params.scope,
            client_key: path_params.client_key,
        })
        .await;

    // Redirect::to(&location);

    match result {
        Ok(res) => {
            info!(
                message = "successfully return oauth2 profile info",
                response = ?res,
            );

            Response::success().payload(Oauth2AuthenticateRes {
                profile: res.profile,
            })
        },
        Err(err) => {
            info!(
                message = "failed to return oauth2 profile info",
                err = ?err,
            );

            Response::fail().error(err.into_error())
        },
    }
}

// fn location(
//     // provider: &Provider,
//     profile: JsonValue,
// ) -> Result<String, url::ParseError> {
//     // let mindly_app_base_url = provider.fetch::<MindlyAppBaseUrl>();

//     let location = mindly_app_base_url
//         .join("/auth/sso/login/")?
//         .join(&refresh_token.to_string())?
//         .to_string();

//     println!("{}", location);

//     Ok(location)
// }
