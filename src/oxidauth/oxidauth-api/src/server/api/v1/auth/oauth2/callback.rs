use axum::{
    Form,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use oxidauth_kernel::{
    JsonValue,
    auth::authenticate_or_register::{
        AuthenticateOrRegisterParams,
        AuthenticateOrRegisterService,
        OAuth2AuthenticateParams,
        OAuth2AuthenticatePathParams,
    },
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use url::Url;
use uuid::Uuid;

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
    run_flow(
        provider,
        path_params.client_key,
        auth_response.code,
        auth_response.state,
        auth_response.scope,
    )
    .await
}

/// Sign in with Apple answers the flow with a form POST to the redirect URI
/// (`response_mode=form_post`, OXA-000072) instead of a query string. The
/// form's `id_token` and first-consent `user` JSON are deliberately ignored:
/// identity is re-derived from the server-side code exchange and a JWKS-
/// verified token — whatever a browser posts is not trusted. An `error`
/// field (user denied consent, account revoked, …) is logged and mapped to
/// the same static error text.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppleCallbackForm {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
}

#[tracing::instrument(name = "oauth2_apple_callback_handler", skip(provider))]
pub async fn handle_form(
    State(provider): State<Provider>,
    Path(path_params): Path<PathParams>,
    Form(auth_response): Form<AppleCallbackForm>,
) -> Response {
    if let Some(error) = &auth_response.error {
        error!(
            message = "apple callback carried an authorization error",
            error = error.as_str(),
            error_description = auth_response
                .error_description
                .as_deref()
                .unwrap_or("none"),
        );

        return ERROR_RESPONSE.into_response();
    }

    let (Some(code), Some(state)) = (auth_response.code.clone(), auth_response.state.clone())
    else {
        return (StatusCode::BAD_REQUEST, "missing field `code` or `state`").into_response();
    };

    run_flow(
        provider,
        path_params.client_key,
        code,
        state,
        auth_response.scope,
    )
    .await
}

async fn run_flow(
    provider: Provider,
    client_key: Uuid,
    code: String,
    state: String,
    scope: Option<String>,
) -> Response {
    let service = provider.fetch_unchecked::<AuthenticateOrRegisterService>();

    let params = {
        let params = OAuth2AuthenticateParams {
            code,
            scope: scope.clone(),
            client_key,
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
        .authenticate_or_register(&AuthenticateOrRegisterParams {
            client_key,
            state,
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
    given_name: Option<String>,
    family_name: Option<String>,
    user_id: Uuid,
) -> Result<Url, url::ParseError> {
    // let mut path = format!(
    //     "/auth/sso/login/{}?email={}&user_id={}",
    //     refresh_token, email, user_id,
    // );

    // if let Some(given_name) = given_name {
    //     path = format!("{}&given_name={}", path, given_name)
    // }

    // if let Some(family_name) = family_name {
    //     path = format!("{}&family_name={}", path, family_name)
    // }

    // base.join(&path)

    let mut url = base.join(&format!("/auth/sso/login/{}", refresh_token))?;

    url.query_pairs_mut()
        .append_pair("email", &email)
        .append_pair("user_id", &user_id.to_string());

    if let Some(given_name) = given_name {
        url.query_pairs_mut()
            .append_pair("given_name", &given_name);
    }

    if let Some(family_name) = family_name {
        url.query_pairs_mut()
            .append_pair("family_name", &family_name);
    }

    Ok(url)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use axum::{
        Router,
        body::Body,
        extract::Request,
        http::{
            StatusCode,
            header::{CONTENT_TYPE, LOCATION},
        },
    };
    use http_body_util::BodyExt;
    use oxidauth_kernel::{
        auth::authenticate_or_register::{
            AuthenticateOrRegisterResponse,
            AuthenticateOrRegisterServiceTrait,
        },
        error::BoxedError,
    };
    use tower::ServiceExt;

    use super::*;

    // Hand-written mock for the one service the handler calls; the provider is
    // keyed by the `AuthenticateOrRegisterService = Arc<dyn …Trait>` alias, so
    // the handler's `fetch_unchecked` picks it up exactly like production DI.
    #[derive(Clone)]
    struct MockAuthenticateOrRegister {
        fail: bool,
        client_base: String,
        refresh_token: Uuid,
        email: String,
        given_name: Option<String>,
        family_name: Option<String>,
        user_id: Uuid,
        captured: Arc<Mutex<Vec<AuthenticateOrRegisterParams>>>,
    }

    impl MockAuthenticateOrRegister {
        fn success(client_base: &str) -> Self {
            Self {
                fail: false,
                client_base: client_base.to_string(),
                refresh_token: Uuid::nil(),
                email: "user@example.com".to_string(),
                given_name: Some("Ada".to_string()),
                family_name: Some("Lovelace".to_string()),
                user_id: Uuid::nil(),
                captured: Arc::default(),
            }
        }

        fn fail() -> Self {
            Self {
                fail: true,
                client_base: "https://app.example.com/".to_string(),
                refresh_token: Uuid::nil(),
                email: "user@example.com".to_string(),
                given_name: None,
                family_name: None,
                user_id: Uuid::nil(),
                captured: Arc::default(),
            }
        }
    }

    #[async_trait]
    impl AuthenticateOrRegisterServiceTrait for MockAuthenticateOrRegister {
        async fn authenticate_or_register(
            &self,
            params: &AuthenticateOrRegisterParams,
        ) -> Result<AuthenticateOrRegisterResponse, BoxedError> {
            self.captured
                .lock()
                .expect("captured")
                .push(AuthenticateOrRegisterParams {
                    client_key: params.client_key,
                    state: params.state.clone(),
                    params: params.params.clone(),
                });

            if self.fail {
                // stands in for the strategy-lookup failure (no authority for
                // client_key, or exchange/profile rejection inside the service)
                return Err("mock: oauth2 strategy lookup failed".into());
            }

            Ok(AuthenticateOrRegisterResponse {
                jwt: "signed.jwt.string".to_string(),
                refresh_token: self.refresh_token,
                client_base: Url::parse(&self.client_base).expect("valid base url"),
                email: self.email.clone(),
                given_name: self.given_name.clone(),
                family_name: self.family_name.clone(),
                user_id: self.user_id,
            })
        }
    }

    // nest the REAL production `oauth2::router()` (not a hand-copied route
    // shape) so a route rename breaks these tests loudly
    fn app(mock: MockAuthenticateOrRegister) -> Router {
        let mut provider = Provider::new();

        provider.store::<AuthenticateOrRegisterService>(Arc::new(mock));

        Router::new()
            .nest("/auth/oauth2", super::super::router())
            .with_state(provider)
    }

    async fn get_response(app: Router, uri: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("should be able to build a request"),
        )
        .await
        .expect("should be able to serve the request")
    }

    async fn post_form(app: Router, key: &str, body: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/auth/oauth2/callback/{key}"))
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from(body.to_owned()))
                .expect("should be able to build a request"),
        )
        .await
        .expect("should be able to serve the request")
    }

    async fn body_text(response: axum::response::Response) -> String {
        let bytes = BodyExt::collect(response.into_body())
            .await
            .expect("should be able to read the response body")
            .to_bytes()
            .to_vec();

        String::from_utf8(bytes).expect("body is utf8")
    }

    fn client_key() -> String {
        Uuid::new_v4().to_string()
    }

    #[tokio::test]
    async fn missing_query_params_are_rejected_with_400() {
        let uri = format!("/auth/oauth2/callback/{}", client_key());
        let response = get_response(app(MockAuthenticateOrRegister::fail()), &uri).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = body_text(response).await;

        assert!(
            body.contains("missing field `code`"),
            "expected the missing-param rejection to name `code`, body was: {body}"
        );
    }

    #[tokio::test]
    async fn missing_state_query_param_is_rejected_with_400() {
        let uri = format!("/auth/oauth2/callback/{}?code=the-code", client_key());
        let response = get_response(app(MockAuthenticateOrRegister::fail()), &uri).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = body_text(response).await;

        assert!(
            body.contains("missing field `state`"),
            "expected the missing-param rejection to name `state`, body was: {body}"
        );
    }

    #[tokio::test]
    async fn malformed_client_key_is_rejected_with_400() {
        let response = get_response(
            app(MockAuthenticateOrRegister::fail()),
            "/auth/oauth2/callback/not-a-uuid",
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = body_text(response).await;

        assert!(
            body.to_lowercase()
                .contains("invalid"),
            "expected the malformed-path rejection to explain the invalid uuid, body was: {body}"
        );
    }

    #[tokio::test]
    async fn service_failure_maps_to_the_static_error_text() {
        let uri = format!(
            "/auth/oauth2/callback/{}?code=the-code&state=the-state",
            client_key()
        );
        let response = get_response(app(MockAuthenticateOrRegister::fail()), &uri).await;

        // BUG(pinned): a failed oauth2 authenticate-or-register answers HTTP 200
        // with plain-text ERROR_RESPONSE — `&str::into_response()` carries no 4xx
        // status, so clients (and hurl) cannot distinguish failure from success
        // without parsing the body.
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_text(response).await, ERROR_RESPONSE);
    }

    #[tokio::test]
    async fn success_redirects_with_refresh_token_email_and_user_id() {
        let mut mock = MockAuthenticateOrRegister::success("https://app.example.com/");
        mock.refresh_token = Uuid::new_v4();
        mock.user_id = Uuid::new_v4();

        let expected_location = format!(
            "https://app.example.com/auth/sso/login/{}?email=user%40example.com&user_id={}&given_name=Ada&family_name=Lovelace",
            mock.refresh_token, mock.user_id,
        );

        let uri = format!(
            "/auth/oauth2/callback/{}?code=the-code&state=the-state",
            client_key()
        );
        let response = get_response(app(mock), &uri).await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers()[LOCATION],
            expected_location,
            "redirect must carry refresh token, percent-encoded email, user id and names"
        );
    }

    #[tokio::test]
    async fn success_redirect_omits_absent_names() {
        let mut mock = MockAuthenticateOrRegister::success("https://app.example.com/");
        mock.given_name = None;
        mock.family_name = None;

        let uri = format!(
            "/auth/oauth2/callback/{}?code=the-code&state=the-state",
            client_key()
        );
        let response = get_response(app(mock), &uri).await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let location = response.headers()[LOCATION]
            .to_str()
            .expect("location is ascii")
            .to_string();

        assert!(
            !location.contains("given_name") && !location.contains("family_name"),
            "absent names must not appear in the redirect: {location}"
        );
    }

    #[tokio::test]
    async fn unbuildable_redirect_url_falls_back_to_the_static_error_text() {
        // client_base is a cannot-be-a-base URL, so gen_redirect_url's join fails
        let mut mock = MockAuthenticateOrRegister::success("about:blank");
        mock.given_name = None;
        mock.family_name = None;

        let uri = format!(
            "/auth/oauth2/callback/{}?code=the-code&state=the-state",
            client_key()
        );
        let response = get_response(app(mock), &uri).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_text(response).await, ERROR_RESPONSE);
    }

    #[tokio::test]
    async fn apple_style_form_post_completes_the_flow_like_the_get_path() {
        // OXA-000072: the same pipeline behind a form_post callback — code
        // and state arrive in the body instead of the query string
        let mock = MockAuthenticateOrRegister::success("https://app.example.com/");
        let captured = mock.captured.clone();

        let response = post_form(
            app(mock),
            &client_key(),
            "code=apple-code&state=apple-state&scope=openid%20email",
        )
        .await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let captured = captured
            .lock()
            .expect("captured");
        let first = captured
            .first()
            .expect("the service was called exactly once");
        assert_eq!(captured.len(), 1);
        assert_eq!(first.state, "apple-state");

        let wire = serde_json::to_value(&first.params).expect("params serialize");
        assert_eq!(wire["code"], "apple-code");
        assert_eq!(wire["scope"], "openid email");
    }

    #[tokio::test]
    async fn form_post_never_forwards_apple_form_id_token_or_user() {
        // anything the browser posts under `id_token`/`user` is untrusted
        // noise: identity is re-derived server-side from the code exchange,
        // so those fields must not reach the service params (OXA-000072)
        let mock = MockAuthenticateOrRegister::success("https://app.example.com/");
        let captured = mock.captured.clone();

        let response = post_form(
            app(mock),
            &client_key(),
            "code=real-code&state=s&id_token=attacker.jwt&user=%7B%22name%22%3A%22evil%22%7D",
        )
        .await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let captured = captured
            .lock()
            .expect("captured");
        let wire = serde_json::to_value(&captured[0].params).expect("params serialize");

        assert_eq!(wire["code"], "real-code");
        assert!(
            !wire
                .to_string()
                .contains("attacker")
                && !wire
                    .to_string()
                    .contains("evil"),
            "form-supplied identity material leaked into the service params: {wire}"
        );
    }

    #[tokio::test]
    async fn form_post_without_code_or_state_is_rejected_with_400() {
        let response = post_form(
            app(MockAuthenticateOrRegister::fail()),
            &client_key(),
            "state=apple-state",
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(body_text(response).await, "missing field `code` or `state`");
    }

    #[tokio::test]
    async fn form_post_carrying_apple_error_never_reaches_the_service() {
        // user denial: logged and mapped to the static error text (200-on-
        // error shape stays pinned to the GET behavior until OXA-000037)
        let mock = MockAuthenticateOrRegister::success("https://app.example.com/");
        let captured = mock.captured.clone();

        let response = post_form(
            app(mock),
            &client_key(),
            "error=user_consented&error_description=nope&state=apple-state",
        )
        .await;

        assert_eq!(body_text(response).await, ERROR_RESPONSE);
        assert!(
            captured
                .lock()
                .expect("captured")
                .is_empty(),
            "a denied authorization must not drive the pipeline"
        );
    }

    #[test]
    fn gen_redirect_url_percent_encodes_query_values() {
        let refresh_token = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let url = gen_redirect_url(
            Url::parse("https://app.example.com/").expect("valid base"),
            refresh_token,
            "ada+labs@example.com".to_string(),
            Some("Ada Lovelace".to_string()),
            Some("Lovelace & Co".to_string()),
            user_id,
        )
        .expect("should be able to build the redirect url");

        assert_eq!(
            url.as_str(),
            format!(
                "https://app.example.com/auth/sso/login/{refresh_token}?email=ada%2Blabs%40example.com&user_id={user_id}&given_name=Ada+Lovelace&family_name=Lovelace+%26+Co"
            )
        );
    }

    #[test]
    fn gen_redirect_url_replaces_base_path_and_query() {
        let refresh_token = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let url = gen_redirect_url(
            Url::parse("https://app.example.com/some/where?left=over").expect("valid base"),
            refresh_token,
            "ada@example.com".to_string(),
            None,
            None,
            user_id,
        )
        .expect("should be able to build the redirect url");

        assert_eq!(
            url.as_str(),
            format!(
                "https://app.example.com/auth/sso/login/{refresh_token}?email=ada%40example.com&user_id={user_id}"
            )
        );
    }
}
