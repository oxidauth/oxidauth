use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
};
use serde::{Deserialize, Serialize};
use tracing::info;
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

#[tracing::instrument(name = "oauth2_authenticate_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(path_params): Path<PathParams>,
    auth_response: Query<OAuth2AuthenticatePathParams>,
) -> impl IntoResponse {
    let service = provider.fetch::<AuthenticateOrRegisterService>();

    let params = {
        let params = OAuth2AuthenticateParams {
            code: auth_response.code.clone(),
            scope: auth_response.scope.clone(),
            client_key: path_params.client_key,
        };

        let params = serde_json::to_value(&params).unwrap();

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

            let location = location(
                res.client_base,
                res.refresh_token,
                res.email,
                res.given_name,
                res.family_name,
                res.user_id,
            );

            Redirect::to(&location.as_str())
        },
        Err(err) => {
            info!(
                message = "oauth2 authenticate or register fail",
                err = ?err,
            );

            println!("REDIRECT LOCATION FAIL");

            // Response::fail().error(err.into_error())
            // todo!()

            Redirect::to("http://app.mindly.localhost/")
        },
    }
}

fn location(
    base: Url,
    refresh_token: Uuid,
    email: String,
    given_name: String,
    family_name: String,
    user_id: Uuid,
) -> String {
    format!(
        "{}auth/sso/login/{}?email={}&given_name={}&family_name={}&user_id={}",
        base,
        refresh_token.to_string(),
        email,
        given_name,
        family_name,
        user_id,
    )
}
