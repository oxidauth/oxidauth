use axum::{Json, extract::State, response::IntoResponse};
use oxidauth_http::{
    Response,
    invitations::delete_invitation::{DeleteInvitationReq, DeleteInvitationRes},
};
use oxidauth_kernel::{
    error::IntoOxidAuthError,
    invitations::delete_invitation::DeleteInvitationService,
};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

pub const PERMISSION: &str = "oxidauth:invitations:delete";

#[tracing::instrument(name = "delete_invitation_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Json(params): Json<DeleteInvitationReq>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<DeleteInvitationService>();

    info!("provided DeleteInvitationService");

    let result = service
        .delete_invitation(&params.invitation)
        .await;

    match result {
        Ok(invitation) => {
            info!(
                message = "successfully deleted invitation",
                invitation = ?invitation,
            );

            Response::success().payload(DeleteInvitationRes { invitation })
        },
        Err(err) => {
            info!(
                message = "failed to delete invitation",
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
            delete_invitation::{DeleteInvitationParams, DeleteInvitationServiceTrait},
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
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::*;

    // See `create_invitation::tests` for the harness rationale (OXA-000002
    // blind spot, callback.rs pattern, REAL `invitations::router()`).

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

    struct MockDeleteInvitation {
        fail: bool,
        calls: Arc<Mutex<Vec<Uuid>>>,
    }

    #[async_trait]
    impl DeleteInvitationServiceTrait for MockDeleteInvitation {
        async fn delete_invitation(
            &self,
            params: &DeleteInvitationParams,
        ) -> Result<Invitation, BoxedError> {
            self.calls
                .lock()
                .expect("log")
                .push(params.id);

            if self.fail {
                return Err("mock: invitation delete failed".into());
            }

            Ok(invitation_fixture(params.id))
        }
    }

    fn invitation_fixture(id: Uuid) -> Invitation {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "user_id": Uuid::new_v4().to_string(),
            "expires_at": "2026-12-31T00:00:00Z",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
        }))
        .expect("valid invitation fixture")
    }

    fn app(fail_service: bool) -> (Router, Arc<Mutex<Vec<Uuid>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));

        let mut provider = Provider::new();

        provider.store::<ListAllPublicKeysService>(Arc::new(MockPublicKeys));
        provider.store::<DeleteInvitationService>(Arc::new(MockDeleteInvitation {
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

    fn call_count(calls: &Mutex<Vec<Uuid>>) -> usize {
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

    fn delete(token: Option<&str>, invitation_id: Uuid, body: &str) -> Request {
        let mut builder = Request::builder()
            .method("DELETE")
            .uri(format!("/invitations/{invitation_id}"))
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
    async fn permitted_delete_returns_the_invitation_and_passes_the_body_id_through() {
        let invitation_id = Uuid::new_v4();
        let token = signed_token(&[PERMISSION]);
        let (app, calls) = app(false);

        let response = app
            .oneshot(delete(
                Some(&token),
                invitation_id,
                &format!(r#"{{"invitation":{{"id":"{invitation_id}"}}}}"#),
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
        assert_eq!(
            calls
                .lock()
                .expect("log")
                .as_slice(),
            &[invitation_id],
            "the body invitation.id must reach the service verbatim"
        );
    }

    #[tokio::test]
    async fn valid_jwt_without_the_permission_is_refused_with_the_success_false_401() {
        let token = signed_token(&["oxidauth:invitations:read"]);
        let (app, calls) = app(false);

        let invitation_id = Uuid::new_v4();
        let response = app
            .oneshot(delete(
                Some(&token),
                invitation_id,
                &format!(r#"{{"invitation":{{"id":"{invitation_id}"}}}}"#),
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
    async fn anonymous_delete_is_refused_with_401_before_the_service_runs() {
        let invitation_id = Uuid::new_v4();
        let (app, calls) = app(false);

        let response = app
            .oneshot(delete(
                None,
                invitation_id,
                &format!(r#"{{"invitation":{{"id":"{invitation_id}"}}}}"#),
            ))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(call_count(&calls), 0);
    }

    #[tokio::test]
    async fn service_failure_maps_to_the_canonical_400_error_envelope() {
        let invitation_id = Uuid::new_v4();
        let token = signed_token(&[PERMISSION]);
        let (app, calls) = app(true);

        let response = app
            .oneshot(delete(
                Some(&token),
                invitation_id,
                &format!(r#"{{"invitation":{{"id":"{invitation_id}"}}}}"#),
            ))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let envelope = json_envelope(response).await;
        assert_eq!(envelope["success"], serde_json::Value::Bool(false));
        assert_eq!(envelope["errors"][0]["name"], "BoxedError");
        assert_eq!(
            envelope["errors"][0]["display"],
            "mock: invitation delete failed"
        );
        assert_eq!(call_count(&calls), 1);
    }

    #[tokio::test]
    async fn a_malformed_granted_permission_maps_to_the_400_parse_error() {
        // The gate's reachable `Err` arm (single `parse_and_validate`): one
        // unparseable granted permission (three colons) short-circuits into
        // `Response::bad_request().error(err.to_string())` before the service.
        let invitation_id = Uuid::new_v4();
        let token = signed_token(&["too:many:colons:here"]);
        let (app, calls) = app(false);

        let response = app
            .oneshot(delete(
                Some(&token),
                invitation_id,
                &format!(r#"{{"invitation":{{"id":"{invitation_id}"}}}}"#),
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
