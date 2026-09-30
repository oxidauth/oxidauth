use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    invitations::accept_invitation::{
        AcceptInvitationBodyReq,
        AcceptInvitationPathReq,
        AcceptInvitationRes,
    },
};
use oxidauth_kernel::error::IntoOxidAuthError;
pub use oxidauth_kernel::invitations::accept_invitation::*;
use provider::Provider;
use tracing::info;

#[tracing::instrument(name = "accept_invitation_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    Path(path): Path<AcceptInvitationPathReq>,
    Json(body): Json<AcceptInvitationBodyReq>,
) -> impl IntoResponse {
    let service = provider.fetch_unchecked::<AcceptInvitationService>();

    info!("provided AcceptInvitationService");

    let result = service
        .accept_invitation(&AcceptInvitationParams {
            invitation_id: path.invitation_id,
            user: body.user,
            user_authority: body.user_authority,
        })
        .await;

    match result {
        Ok(user) => {
            info!(
                message = "successfully accepted invitation",
                user = ?user,
            );

            Response::success().payload(AcceptInvitationRes { user })
        },
        Err(err) => {
            info!(
                message = "failed to accept invitation",
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
        http::{StatusCode, header::CONTENT_TYPE},
    };
    use http_body_util::BodyExt;
    use oxidauth_kernel::{error::BoxedError, users::User};
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::*;

    // See `create_invitation::tests` for the harness rationale. This route is
    // the invitation-accept leg (OXA-000002 blind spot) and is deliberately
    // gateless — invitees hold no jwt; the pins below show the merge of path
    // id + body params into `AcceptInvitationParams` and the canonical arms.

    #[derive(Clone, Debug, PartialEq)]
    struct Received {
        invitation_id: Uuid,
        username: String,
        client_key: Uuid,
    }

    struct MockAcceptInvitation {
        fail: bool,
        calls: Arc<Mutex<Vec<Received>>>,
    }

    #[async_trait]
    impl AcceptInvitationServiceTrait for MockAcceptInvitation {
        async fn accept_invitation(
            &self,
            params: &AcceptInvitationParams,
        ) -> Result<User, BoxedError> {
            self.calls
                .lock()
                .expect("log")
                .push(Received {
                    invitation_id: params.invitation_id,
                    username: params.user.username.clone(),
                    client_key: params
                        .user_authority
                        .client_key,
                });

            if self.fail {
                return Err("mock: invitation expired".into());
            }

            Ok(user_fixture(params.invitation_id, &params.user.username))
        }
    }

    fn user_fixture(id: Uuid, username: &str) -> User {
        serde_json::from_value(serde_json::json!({
            "id": id.to_string(),
            "kind": "human",
            "status": "enabled",
            "username": username,
            "email": null,
            "first_name": null,
            "last_name": null,
            "profile": {},
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
        }))
        .expect("valid user fixture")
    }

    fn app(fail_service: bool) -> (Router, Arc<Mutex<Vec<Received>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));

        let mut provider = Provider::new();

        provider.store::<AcceptInvitationService>(Arc::new(MockAcceptInvitation {
            fail: fail_service,
            calls: Arc::clone(&calls),
        }));

        (
            Router::new()
                .nest(
                    "/invitations",
                    crate::server::api::v1::invitations::router(),
                )
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

    fn post(invitation_id: Uuid, body: &str) -> Request {
        Request::builder()
            .method("POST")
            .uri(format!("/invitations/{invitation_id}"))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
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

    fn body(client_key: Uuid) -> String {
        format!(
            r#"{{"user":{{"username":"invitee","email":"ada@example.com"}},"user_authority":{{"client_key":"{client_key}","params":{{}}}}}}"#
        )
    }

    #[tokio::test]
    async fn accepting_an_invitation_answers_200_with_the_new_user() {
        let invitation_id = Uuid::new_v4();
        let (app, calls) = app(false);

        let response = app
            .oneshot(post(invitation_id, &body(Uuid::new_v4())))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);

        let envelope = json_envelope(response).await;
        assert_eq!(envelope["success"], serde_json::Value::Bool(true));
        assert_eq!(envelope["payload"]["user"]["username"], "invitee");
        assert_eq!(
            envelope["payload"]["user"]["id"],
            invitation_id.to_string(),
            "the canonical success shape is `{{success:true, payload:{{user:…}}}}`"
        );
        assert_eq!(call_count(&calls), 1);
    }

    #[tokio::test]
    async fn path_id_and_body_fields_merge_into_the_service_params_verbatim() {
        let invitation_id = Uuid::new_v4();
        let client_key = Uuid::new_v4();
        let (app, calls) = app(false);

        let response = app
            .oneshot(post(invitation_id, &body(client_key)))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            received(&calls),
            vec![Received {
                invitation_id,
                username: "invitee".to_string(),
                client_key,
            }],
            "invitation_id must come from the path, user/user_authority from the body"
        );
    }

    #[tokio::test]
    async fn service_failure_maps_to_the_canonical_400_error_envelope() {
        let (app, calls) = app(true);

        let response = app
            .oneshot(post(Uuid::new_v4(), &body(Uuid::new_v4())))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let envelope = json_envelope(response).await;
        assert_eq!(envelope["success"], serde_json::Value::Bool(false));
        assert_eq!(envelope["errors"][0]["name"], "BoxedError");
        assert_eq!(envelope["errors"][0]["display"], "mock: invitation expired");
        assert_eq!(call_count(&calls), 1);
    }

    #[tokio::test]
    async fn a_body_missing_user_authority_is_rejected_with_422_before_the_service_runs() {
        let (app, calls) = app(false);

        let response = app
            .oneshot(post(Uuid::new_v4(), r#"{"user":{"username":"invitee"}}"#))
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
            body_text.contains("missing field `user_authority`"),
            "expected the rejection to name the missing field, body was: {body_text}"
        );
        assert_eq!(call_count(&calls), 0);
    }
}
