use axum::{Json, extract::State, response::IntoResponse};
use oxidauth_http::{Response, invitations::create_invitation::CreateInvitationReq};
use oxidauth_kernel::{
    error::IntoOxidAuthError,
    invitations::create_invitation::CreateInvitationService,
};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

pub const PERMISSION: &str = "oxidauth:invitations:create";

#[tracing::instrument(name = "create_invitation_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Json(params): Json<CreateInvitationReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<CreateInvitationService>();

    info!("provided CreateInvitationService");

    let result = service
        .create_invitation(&params.invitation)
        .await;

    match result {
        Ok(invitation) => {
            info!(
                message = "successfully created invitation",
                invitation = ?invitation,
            );

            Response::success().payload(invitation)
        },
        Err(err) => {
            info!(
                message = "failed to create invitation",
                err = ?err,
            );

            Response::bad_request().error(err.into_error())
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, LazyLock, Mutex};

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
        error::BoxedError,
        invitations::{
            Invitation,
            create_invitation::{
                CreateInvitationParams,
                CreateInvitationResponse,
                CreateInvitationServiceTrait,
            },
        },
        jwt::{EntitlementsEncoding, Jwt},
        public_keys::{
            PublicKey,
            list_all_public_keys::{
                ListAllPublicKeys,
                ListAllPublicKeysService,
                ListAllPublicKeysServiceTrait,
            },
        },
        rsa::KeyPair,
        users::User,
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::*;

    // callback.rs pattern + permission_extractor.rs jwt harness, driving the
    // REAL `invitations::router()`. Closes the OXA-000002 zero-e2e blind spot
    // at the handler tier: create/find/accept/delete each get their canonical
    // arms, gate, and params pass-through pinned in unit tests.

    static KEYPAIR: LazyLock<KeyPair> =
        LazyLock::new(|| KeyPair::new().expect("should be able to generate a test keypair"));

    struct MockPublicKeys;

    #[async_trait]
    impl ListAllPublicKeysServiceTrait for MockPublicKeys {
        async fn list_all_public_keys(
            &self,
            _params: &ListAllPublicKeys,
        ) -> Result<Vec<PublicKey>, BoxedError> {
            let at = serde_json::from_str("\"2026-01-01T00:00:00Z\"")
                .expect("should be able to parse a fixed timestamp");

            Ok(vec![PublicKey {
                id: Uuid::nil(),
                public_key: String::from_utf8(KEYPAIR.public.clone()).expect("pem is utf8"),
                created_at: at,
                updated_at: at,
            }])
        }
    }

    #[derive(Clone)]
    struct Received {
        id: Option<Uuid>,
        username: String,
    }

    struct MockCreateInvitation {
        fail: bool,
        invitation_id: Uuid,
        calls: Arc<Mutex<Vec<Received>>>,
    }

    #[async_trait]
    impl CreateInvitationServiceTrait for MockCreateInvitation {
        async fn create_invitation(
            &self,
            params: &CreateInvitationParams,
        ) -> Result<CreateInvitationResponse, BoxedError> {
            self.calls
                .lock()
                .expect("log")
                .push(Received {
                    id: params.id,
                    username: params.user.username.clone(),
                });

            if self.fail {
                return Err("mock: invitation insert failed".into());
            }

            Ok(CreateInvitationResponse {
                invitation: invitation_fixture(self.invitation_id, Uuid::new_v4()),
                user: user_fixture(Uuid::new_v4(), &params.user.username),
            })
        }
    }

    fn invitation_fixture(id: Uuid, user_id: Uuid) -> Invitation {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "user_id": user_id,
            "expires_at": "2026-12-31T00:00:00Z",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
        }))
        .expect("valid invitation fixture")
    }

    fn user_fixture(id: Uuid, username: &str) -> User {
        serde_json::from_value(serde_json::json!({
            "id": id.to_string(),
            "kind": "human",
            "status": "invited",
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

    fn app(fail_service: bool) -> (Router, Uuid, Arc<Mutex<Vec<Received>>>) {
        let invitation_id = Uuid::new_v4();
        let calls = Arc::new(Mutex::new(Vec::new()));

        let mut provider = Provider::new();

        provider.store::<ListAllPublicKeysService>(Arc::new(MockPublicKeys));
        provider.store::<CreateInvitationService>(Arc::new(MockCreateInvitation {
            fail: fail_service,
            invitation_id,
            calls: Arc::clone(&calls),
        }));

        (
            Router::new()
                .nest(
                    "/invitations",
                    crate::server::api::v1::invitations::router(),
                )
                .with_state(provider),
            invitation_id,
            calls,
        )
    }

    fn call_count(calls: &Mutex<Vec<Received>>) -> usize {
        calls
            .lock()
            .expect("log")
            .len()
    }

    fn signed_token(permissions: &[&str]) -> String {
        let permissions: Vec<String> = permissions
            .iter()
            .map(ToString::to_string)
            .collect();

        Jwt::builder()
            .with_subject(Uuid::new_v4())
            .with_expires_in(std::time::Duration::from_secs(600))
            .with_entitlements(EntitlementsEncoding::Txt, &permissions)
            .build()
            .expect("should be able to build a jwt")
            .encode(&KEYPAIR.private)
            .expect("should be able to sign the jwt")
    }

    fn post(token: Option<&str>, body: &str) -> Request {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/invitations")
            .header(CONTENT_TYPE, "application/json");

        if let Some(token) = token {
            builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
        }

        builder
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

    #[tokio::test]
    async fn permitted_create_returns_the_invitation_and_the_invited_user() {
        let token = signed_token(&[PERMISSION]);
        let (app, invitation_id, calls) = app(false);

        let response = app
            .oneshot(post(
                Some(&token),
                r#"{"invitation":{"user":{"username":"invitee"}}}"#,
            ))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);

        let envelope = json_envelope(response).await;
        assert_eq!(envelope["success"], serde_json::Value::Bool(true));
        assert_eq!(
            envelope["payload"]["invitation"]["id"],
            invitation_id.to_string()
        );
        assert_eq!(envelope["payload"]["user"]["username"], "invitee");
        assert_eq!(envelope["payload"]["user"]["status"], "invited");
        assert_eq!(call_count(&calls), 1);
    }

    #[tokio::test]
    async fn the_invitation_object_passes_through_to_the_service_verbatim() {
        let requested_id = Uuid::new_v4();
        let token = signed_token(&[PERMISSION]);
        let (app, _invitation_id, calls) = app(false);

        let response = app
            .oneshot(post(
                Some(&token),
                &format!(
                    r#"{{"invitation":{{"id":"{requested_id}","user":{{"username":"ada"}}}}}}"#
                ),
            ))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);

        let calls = calls.lock().expect("log");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, Some(requested_id));
        assert_eq!(calls[0].username, "ada");
    }

    #[tokio::test]
    async fn anonymous_create_is_refused_with_401_before_the_service_runs() {
        let (app, _invitation_id, calls) = app(false);

        let response = app
            .oneshot(post(
                None,
                r#"{"invitation":{"user":{"username":"invitee"}}}"#,
            ))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(call_count(&calls), 0);
    }

    #[tokio::test]
    async fn valid_jwt_without_the_permission_is_refused_with_the_success_false_401() {
        let token = signed_token(&["oxidauth:invitations:read"]);
        let (app, _invitation_id, calls) = app(false);

        let response = app
            .oneshot(post(
                Some(&token),
                r#"{"invitation":{"user":{"username":"invitee"}}}"#,
            ))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            json_envelope(response).await,
            serde_json::json!({ "success": false })
        );
        assert_eq!(call_count(&calls), 0);
    }

    #[tokio::test]
    async fn service_failure_maps_to_the_canonical_400_error_envelope() {
        let token = signed_token(&[PERMISSION]);
        let (app, _invitation_id, calls) = app(true);

        let response = app
            .oneshot(post(
                Some(&token),
                r#"{"invitation":{"user":{"username":"invitee"}}}"#,
            ))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let envelope = json_envelope(response).await;
        assert_eq!(envelope["success"], serde_json::Value::Bool(false));
        assert_eq!(envelope["errors"][0]["name"], "BoxedError");
        assert_eq!(
            envelope["errors"][0]["display"],
            "mock: invitation insert failed"
        );
        assert_eq!(call_count(&calls), 1);
    }

    #[tokio::test]
    async fn a_malformed_granted_permission_maps_to_the_400_parse_error() {
        let token = signed_token(&["too:many:colons:here"]);
        let (app, _invitation_id, calls) = app(false);

        let response = app
            .oneshot(post(
                Some(&token),
                r#"{"invitation":{"user":{"username":"invitee"}}}"#,
            ))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            json_envelope(response).await,
            serde_json::json!({ "success": false, "errors": ["invalid permission"] })
        );
        assert_eq!(call_count(&calls), 0);
    }
}
