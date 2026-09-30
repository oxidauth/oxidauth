use axum::{Json, extract::State, response::IntoResponse};
use oxidauth_http::Response;
use oxidauth_kernel::auth::username_password::update_password::{
    UpdatePasswordParams,
    UpdatePasswordResponse,
    UpdatePasswordService,
};

use crate::provider::Provider;

#[axum::debug_handler]
#[tracing::instrument(name = "username_password_update_password_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<UpdatePasswordParams>,
) -> impl IntoResponse {
    let service = provider.fetch_unchecked::<UpdatePasswordService>();

    let result = service
        .update_password(&params)
        .await;

    match result {
        Ok(_) => Response::success().payload(UpdatePasswordResponse { success: true }),
        Err(_) => Response::success().payload(UpdatePasswordResponse { success: false }),
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
    use oxidauth_kernel::{
        auth::username_password::update_password::UpdatePasswordServiceTrait,
        error::BoxedError,
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::*;

    // callback.rs pattern: hand-written mock for the one service the handler
    // calls, registered under the `UpdatePasswordService = Arc<dyn …Trait>`
    // alias so the handler's `fetch_unchecked` picks it up exactly like
    // production DI; the REAL `username_password::router()` is mounted at its
    // production path so a route rename breaks these tests loudly.

    #[derive(Clone)]
    struct Received {
        code: String,
        username: String,
        client_key: Uuid,
        password: String,
        password_conf: String,
    }

    struct MockUpdatePassword {
        // `None` answers Ok with the given inner flag; `Some(msg)` answers Err
        outcome: Result<bool, &'static str>,
        received: Arc<Mutex<Vec<Received>>>,
    }

    #[async_trait]
    impl UpdatePasswordServiceTrait for MockUpdatePassword {
        async fn update_password(
            &self,
            params: &UpdatePasswordParams,
        ) -> Result<UpdatePasswordResponse, BoxedError> {
            self.received
                .lock()
                .expect("log")
                .push(Received {
                    code: params.code.clone(),
                    username: params.username.clone(),
                    client_key: params.client_key,
                    password: params.password.clone(),
                    password_conf: params.password_conf.clone(),
                });

            match self.outcome {
                Ok(success) => Ok(UpdatePasswordResponse { success }),
                Err(msg) => Err(msg.into()),
            }
        }
    }

    fn app(outcome: Result<bool, &'static str>) -> (Router, Arc<Mutex<Vec<Received>>>) {
        let received = Arc::new(Mutex::new(Vec::new()));

        let mut provider = Provider::new();

        provider.store::<UpdatePasswordService>(Arc::new(MockUpdatePassword {
            outcome,
            received: Arc::clone(&received),
        }));

        (
            Router::new()
                .nest("/auth/username_password", super::super::router())
                .with_state(provider),
            received,
        )
    }

    fn body(client_key: Uuid) -> String {
        format!(
            r#"{{"code":"123456","username":"ada","client_key":"{client_key}","password":"new-pw","password_conf":"new-pw"}}"#
        )
    }

    fn post(body: String) -> Request {
        Request::builder()
            .method("POST")
            .uri("/auth/username_password/update_password")
            .header(CONTENT_TYPE, "application/json")
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

    fn call_log(received: &Mutex<Vec<Received>>) -> Vec<Received> {
        received
            .lock()
            .expect("log")
            .clone()
    }

    #[tokio::test]
    async fn accepted_password_change_answers_200_with_success_true_payload() {
        let client_key = Uuid::new_v4();
        let (app, received) = app(Ok(true));

        let response = app
            .oneshot(post(body(client_key)))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_envelope(response).await,
            serde_json::json!({ "success": true, "payload": { "success": true } })
        );

        let calls = call_log(&received);
        assert_eq!(calls.len(), 1);

        // params pass-through: the raw JSON body is handed to the service
        // verbatim — the wire contract keeps the passwords raw (kernel
        // update_password::tests pins that only `Debug` masks them)
        let call = &calls[0];
        assert_eq!(call.code, "123456");
        assert_eq!(call.username, "ada");
        assert_eq!(call.client_key, client_key);
        assert_eq!(call.password, "new-pw");
        assert_eq!(call.password_conf, "new-pw");
    }

    #[tokio::test]
    async fn the_route_is_gateless_an_ignored_bearer_jwt_still_reaches_the_service() {
        // Deviation pin (OXA-000059 census): unlike every sibling CRUD route,
        // this handler carries no `ExtractJwt`/`ExtractEntitlements` gate —
        // even a garbage bearer token is never inspected, and the service runs.
        let (app, received) = app(Ok(true));

        let request = Request::builder()
            .method("POST")
            .uri("/auth/username_password/update_password")
            .header(AUTHORIZATION, "Bearer not-a-jwt")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body(Uuid::new_v4())))
            .expect("should be able to build a request");

        let response = app
            .oneshot(request)
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(call_log(&received).len(), 1);
    }

    #[tokio::test]
    async fn service_failure_still_answers_200_with_payload_success_false() {
        let (app, received) = app(Err("simulated outage: pool exhausted"));

        let response = app
            .oneshot(post(body(Uuid::new_v4())))
            .await
            .expect("should be able to serve the request");

        // BUG(pinned): both arms answer HTTP 200 (OXA-000059 census /
        // OXA-000042) — an infra outage and a business refusal are
        // indistinguishable from a success at the status line; only the
        // fabricated-looking `payload.success` flag differs. OXA-000042
        // Step 3 (outage→5xx, refusal→4xx) flips this assertion.
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_envelope(response).await,
            serde_json::json!({ "success": true, "payload": { "success": false } })
        );
        assert_eq!(call_log(&received).len(), 1);
    }

    #[tokio::test]
    async fn a_service_ok_with_success_false_is_discarded_and_fabricated_true() {
        let (app, received) = app(Ok(false));

        let response = app
            .oneshot(post(body(Uuid::new_v4())))
            .await
            .expect("should be able to serve the request");

        // BUG(pinned): the Ok arm matches `Ok(_)` and fabricates
        // `{success: true}` regardless of the service's own flag
        // (OXA-000042's original finding); the service value never reaches
        // the wire. OXA-000042 Step 3 flips this assertion.
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_envelope(response).await,
            serde_json::json!({ "success": true, "payload": { "success": true } })
        );
        assert_eq!(call_log(&received).len(), 1);
    }

    #[tokio::test]
    async fn malformed_body_is_rejected_with_422_before_the_service_runs() {
        let (app, received) = app(Ok(true));

        let response = app
            .oneshot(post(
                r#"{"code":"123456","username":"ada","client_key":"1b4e28ba-2fa1-11d2-883f-0016d3cca9b9","password":"new-pw"}"#
                    .to_string(),
            ))
            .await
            .expect("should be able to serve the request");

        // axum 0.8 `Json` rejects a syntactically-valid body that fails to
        // deserialise with 422 (the extractors' auth rejections are the 401s;
        // the handler's own error arms are the 400s).
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
            body_text.contains("missing field `password_conf`"),
            "expected the rejection to name the missing field, body was: {body_text}"
        );
        assert!(
            call_log(&received).is_empty(),
            "the service must not run for a body that never deserialised"
        );
    }
}
