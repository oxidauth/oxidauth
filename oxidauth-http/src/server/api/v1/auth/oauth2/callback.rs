use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use oxidauth_kernel::{auth::oauth2::authenticate::*, error::IntoOxidAuthError};

use crate::{provider::Provider, response::Response};

pub type OAuth2CallbackReq = OAuth2AuthenticateParams;
pub type OAuth2CallbackRes = OAuth2AuthenticateResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuth2AuthenticatePathParams {
    pub client_key: Uuid,
}

#[tracing::instrument(name = "oauth2_authenticate_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(path_params): Path<OAuth2AuthenticatePathParams>,
    auth_response: Query<OAuth2CallbackReq>,
) -> impl IntoResponse {
    let service = provider.fetch::<OAuth2AuthenticateService>();

    let result = service
        .call(&OAuth2AuthenticateParams {
            code: auth_response.code.clone(),
            scope: auth_response.scope.clone(),
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

            Response::success().payload(OAuth2CallbackRes {
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
