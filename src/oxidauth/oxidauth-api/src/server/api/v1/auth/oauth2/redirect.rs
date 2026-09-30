use axum::{Json, extract::State, response::IntoResponse};
use oxidauth_http::{Response, auth::oauth2::redirect::Oauth2RedirectRes};
use oxidauth_kernel::{auth::oauth2::redirect::*, error::IntoOxidAuthError};
use tracing::info;

use crate::provider::Provider;

#[tracing::instrument(name = "oauth2_redirect_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<Oauth2RedirectParams>,
) -> impl IntoResponse {
    let service = provider.fetch_unchecked::<Oauth2RedirectService>();

    let result = service
        .oauth2_redirect(&params)
        .await;

    match result {
        Ok(res) => {
            info!(
                message = "successfully return oauth2 redirect url",
                response = ?res,
            );

            Response::success().payload(Oauth2RedirectRes {
                redirect_url: res.redirect_url,
            })
        },
        Err(err) => {
            info!(
                message = "failed to return oauth2 redirect url",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
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
            header::{AUTHORIZATION, CONTENT_TYPE},
        },
    };
    use http_body_util::BodyExt;
    use oxidauth_kernel::error::BoxedError;
    use tower::ServiceExt;
    use url::Url;
    use uuid::Uuid;

    use super::*;

    // callback.rs pattern: mock the one service the handler fetches, register
    // it under the `Oauth2RedirectService = Arc<dyn …Trait>` alias, drive the
    // REAL `oauth2::router()` at its production path. This route has no
    // permission gate (deviation census) — the pins below show even a garbage
    // bearer token is never inspected.

    #[derive(Clone, Debug, PartialEq)]
    struct Received {
        client_key: Uuid,
        email: Option<String>,
    }

    struct MockOauth2Redirect {
        fail: bool,
        calls: Arc<Mutex<Vec<Received>>>,
    }

    #[async_trait]
    impl Oauth2RedirectServiceTrait for MockOauth2Redirect {
        async fn oauth2_redirect(
            &self,
            params: &Oauth2RedirectParams,
        ) -> Result<Oauth2RedirectResponse, BoxedError> {
            self.calls
                .lock()
                .expect("log")
                .push(Received {
                    client_key: params.client_key,
                    email: params.email.clone(),
                });

            if self.fail {
                return Err("mock: no authority for client_key".into());
            }

            Ok(Oauth2RedirectResponse {
                redirect_url: Url::parse("https://app.example.com/oauth2/authorize?state=xyz")
                    .expect("valid redirect url"),
            })
        }
    }

    fn app(fail_service: bool) -> (Router, Arc<Mutex<Vec<Received>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));

        let mut provider = Provider::new();

        provider.store::<Oauth2RedirectService>(Arc::new(MockOauth2Redirect {
            fail: fail_service,
            calls: Arc::clone(&calls),
        }));

        (
            Router::new()
                .nest("/auth/oauth2", super::super::router())
                .with_state(provider),
            calls,
        )
    }

    fn call_count(calls: &Mutex<Vec<Received>>) -> usize {
        calls
            .lock()
            .expect("log")
            .len()
    }

    fn received(calls: &Mutex<Vec<Received>>) -> Vec<Received> {
        calls
            .lock()
            .expect("log")
            .clone()
    }

    fn post(body: String, bearer: Option<&str>) -> Request {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/auth/oauth2/redirect")
            .header(CONTENT_TYPE, "application/json");

        if let Some(bearer) = bearer {
            builder = builder.header(AUTHORIZATION, format!("Bearer {bearer}"));
        }

        builder
            .body(Body::from(body))
            .expect("should be able to build a request")
    }

    async fn json_envelope(response: axum::response::Response) -> serde_json::Value {
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("should be able to collect the body")
            .to_bytes();

        serde_json::from_slice(&bytes).expect("the body must be the JSON envelope")
    }

    #[tokio::test]
    async fn success_answers_200_with_the_redirect_url_payload() {
        let body = format!(
            r#"{{"client_key":"{}","email":"ada@example.com"}}"#,
            Uuid::new_v4()
        );
        let (app, calls) = app(false);

        let response = app
            .oneshot(post(body, None))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_envelope(response).await,
            serde_json::json!({
                "success": true,
                "payload": {
                    "redirect_url": "https://app.example.com/oauth2/authorize?state=xyz",
                },
            })
        );
        assert_eq!(call_count(&calls), 1);
    }

    #[tokio::test]
    async fn client_key_and_email_pass_through_and_absent_email_is_none() {
        let client_key = Uuid::new_v4();
        let (app, calls) = app(false);

        let with_email = format!(r#"{{"client_key":"{client_key}","email":"ada@example.com"}}"#);
        let response = app
            .clone()
            .oneshot(post(with_email, None))
            .await
            .expect("should be able to serve the request");
        assert_eq!(response.status(), StatusCode::OK);

        let without_email = format!(r#"{{"client_key":"{client_key}"}}"#);
        let response = app
            .oneshot(post(without_email, None))
            .await
            .expect("should be able to serve the request");
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            received(&calls),
            vec![
                Received {
                    client_key,
                    email: Some("ada@example.com".to_string()),
                },
                Received {
                    client_key,
                    email: None,
                },
            ],
            "the optional email must pass through as Some/None, client_key verbatim"
        );
    }

    #[tokio::test]
    async fn the_route_is_gateless_an_ignored_bearer_jwt_still_reaches_the_service() {
        let (app, calls) = app(false);

        let body = format!(r#"{{"client_key":"{}"}}"#, Uuid::new_v4());
        let response = app
            .oneshot(post(body, Some("not-a-jwt")))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            call_count(&calls),
            1,
            "the handler never inspects the Authorization header (no gate)"
        );
    }

    #[tokio::test]
    async fn service_failure_maps_to_the_canonical_400_error_envelope() {
        let body = format!(r#"{{"client_key":"{}"}}"#, Uuid::new_v4());
        let (app, calls) = app(true);

        let response = app
            .oneshot(post(body, None))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let envelope = json_envelope(response).await;
        assert_eq!(envelope["success"], serde_json::Value::Bool(false));
        assert_eq!(envelope["errors"][0]["name"], "BoxedError");
        assert_eq!(
            envelope["errors"][0]["display"],
            "mock: no authority for client_key"
        );
        assert_eq!(call_count(&calls), 1);
    }

    #[tokio::test]
    async fn missing_client_key_is_rejected_with_422_before_the_service_runs() {
        let (app, calls) = app(false);

        let response = app
            .oneshot(post(r#"{"email":"ada@example.com"}"#.to_string(), None))
            .await
            .expect("should be able to serve the request");

        // axum 0.8 `Json` rejects a syntactically-valid body that fails to
        // deserialise with 422 — the service must still never run.
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body_text = String::from_utf8(
            response
                .into_body()
                .collect()
                .await
                .expect("should be able to read the body")
                .to_bytes()
                .to_vec(),
        )
        .expect("body is utf8");

        assert!(
            body_text.contains("missing field `client_key`"),
            "expected the rejection to name the missing field, body was: {body_text}"
        );
        assert_eq!(
            call_count(&calls),
            0,
            "the service must not run for a body that never deserialised"
        );
    }
}
