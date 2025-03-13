use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use url::Url;
use uuid::Uuid;

use oxidauth_kernel::{
    JsonValue,
    auth::authenticate_or_register::{
        AuthenticateOrRegisterParams, AuthenticateOrRegisterService, OAuth2AuthenticateParams,
        OAuth2AuthenticatePathParams,
    },
};

use crate::provider::Provider;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PathParams {
    pub client_key: Uuid,
}

pub const ERROR_RESPONSE: &str =
    "OAuth2 Error - unable to authenticate. Contact support for assistance.";

#[tracing::instrument(name = "oauth2_authenticate_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(path_params): Path<PathParams>,
    Query(auth_response): Query<OAuth2AuthenticatePathParams>,
) -> Response {
    let service = provider.fetch::<AuthenticateOrRegisterService>();

    let params = {
        let params = OAuth2AuthenticateParams {
            code: auth_response.code.clone(),
            scope: auth_response.scope.clone(),
            client_key: path_params.client_key,
        };

        let params = match serde_json::to_value(&params) {
            Ok(params) => params,
            Err(err) => {
                error!(
                    message = "oauth2 authenticate or register params fail",
                    err = ?err,
                );

                return ERROR_RESPONSE.into_response();
            },
        };

        JsonValue::new(params)
    };

    let result = service
        .call(&AuthenticateOrRegisterParams {
            client_key: path_params.client_key,
            params,
        })
        .await;

    match result {
        Ok(res) => {
            info!(
                message = "oauth2 authenticate or register success",
                response = ?res,
            );

            let location = match gen_redirect_url(
                res.client_base,
                res.refresh_token,
                res.email,
                res.given_name,
                res.family_name,
                res.user_id,
            ) {
                Ok(location) => location,
                Err(err) => {
                    error!(
                        message = "oauth2 authenticate or register params fail",
                        err = ?err,
                    );

                    return ERROR_RESPONSE.into_response();
                },
            };

            Redirect::to(location.as_str()).into_response()
        },
        Err(err) => {
            error!(
                message = "oauth2 authenticate or register fail",
                err = ?err,
            );

            ERROR_RESPONSE.into_response()
        },
    }
}

fn gen_redirect_url(
    base: Url,
    refresh_token: Uuid,
    email: String,
    given_name: String,
    family_name: String,
    user_id: Uuid,
) -> Result<Url, url::ParseError> {
    let path = format!(
        "/auth/sso/login/{}?email={}&given_name={}&family_name={}&user_id={}",
        refresh_token, email, given_name, family_name, user_id,
    );

    base.join(&path)
}
