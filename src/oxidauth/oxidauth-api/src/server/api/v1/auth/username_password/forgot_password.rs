use axum::{Json, extract::State, response::IntoResponse};
use oxidauth_http::Response;
use oxidauth_kernel::auth::username_password::forgot_password::{
    ForgotPasswordParams,
    ForgotPasswordResponse,
    ForgotPasswordService,
};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

/// OXA-000005 Step 1: this route used to be anonymous — a raw `curl` could
/// mint any user's live TOTP code and wipe their refresh tokens. Callers must
/// now present a jwt whose entitlements satisfy this permission (the admin
/// wildcard `oxidauth:**:**` covers it); the permission is seeded by the
/// bootstrap for fresh installs, upgrades must add the string to the
/// permissions tree and grant it to support roles.
pub const PERMISSION: &str = "oxidauth:auth:forgot_password";

#[tracing::instrument(name = "username_password_forgot_password_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Json(params): Json<ForgotPasswordParams>,
) -> impl IntoResponse {
    match parse_and_validate(PERMISSION, &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, PERMISSION),
        Ok(false) => {
            warn!("{:?} doesn't have {}", jwt.sub, PERMISSION);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<ForgotPasswordService>();

    let result = service
        .forgot_password(&params)
        .await;

    match result {
        Ok(res) => Response::success().payload(ForgotPasswordResponse { code: res.code }),
        Err(err) => {
            // OXA-000005 Step 1: blind the failure paths. Unknown users,
            // users without a TOTP secret, and delete failures were 400
            // envelopes embedding the raw sqlx `Display` — an existence +
            // enrollment oracle that leaked database internals. The real
            // error is logged for operators; callers only ever see the
            // generic success.
            warn!(
                message = "forgot_password failed; responding with generic success",
                err = ?err,
            );

            Response::success()
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
        auth::username_password::forgot_password::ForgotPasswordServiceTrait,
        error::BoxedError,
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

    // Same harness shape as `middleware::permission_extractor::tests`: one
    // RSA keypair signs real jwts that the mocked public-key service lets the
    // real `ExtractJwt`/`ExtractEntitlements` extractors verify, so the gate
    // is exercised exactly as production runs it.

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

    struct MockForgotPassword {
        fail: bool,
        calls: Arc<Mutex<Vec<Uuid>>>,
    }

    #[async_trait]
    impl ForgotPasswordServiceTrait for MockForgotPassword {
        async fn forgot_password(
            &self,
            params: &ForgotPasswordParams,
        ) -> Result<ForgotPasswordResponse, BoxedError> {
            self.calls
                .lock()
                .expect("log")
                .push(params.user_id);

            if self.fail {
                return Err("simulated sqlx error: database row not found".into());
            }

            Ok(ForgotPasswordResponse {
                code: "123456".to_owned(),
            })
        }
    }

    fn provider_with(fail_service: bool) -> (Provider, Arc<Mutex<Vec<Uuid>>>) {
        let mut provider = Provider::new();

        provider.store::<ListAllPublicKeysService>(Arc::new(MockPublicKeys));

        let calls = Arc::new(Mutex::new(Vec::new()));

        provider.store::<ForgotPasswordService>(Arc::new(MockForgotPassword {
            fail: fail_service,
            calls: Arc::clone(&calls),
        }));

        (provider, calls)
    }

    fn call_count(calls: &Mutex<Vec<Uuid>>) -> usize {
        calls
            .lock()
            .expect("log")
            .len()
    }

    fn received(calls: &Mutex<Vec<Uuid>>) -> Vec<Uuid> {
        calls
            .lock()
            .expect("log")
            .clone()
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

    fn post(token: Option<&str>) -> Request {
        post_user(token, Uuid::new_v4())
    }

    fn post_user(token: Option<&str>, user_id: Uuid) -> Request {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/forgot_password")
            .header(CONTENT_TYPE, "application/json");

        if let Some(token) = token {
            builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
        }

        builder
            .body(Body::from(format!(r#"{{"user_id":"{user_id}"}}"#)))
            .expect("should be able to build a request")
    }

    fn app(fail_service: bool) -> (Router, Arc<Mutex<Vec<Uuid>>>) {
        let (provider, calls) = provider_with(fail_service);

        (
            crate::server::api::v1::auth::username_password::router().with_state(provider),
            calls,
        )
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
    async fn anonymous_post_is_refused_with_401_before_the_service_runs() {
        // The OXA-000005 regression: the bare-`curl` leak — POST {user_id}
        // with no Authorization — used to return 200 with a live TOTP code
        // and wipe the victim's refresh tokens. The `ExtractJwt` extractor
        // now rejects anonymously with a bodyless 401 (pinned bodyless at
        // `extractor_rejection_over_a_route_is_a_bodyless_401`), and the use
        // case — code minting AND token revocation — is never reached.
        let (app, calls) = app(false);

        let response = app
            .oneshot(post(None))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            call_count(&calls),
            0,
            "the forgot_password use case must never run for anonymous callers"
        );
    }

    #[tokio::test]
    async fn valid_jwt_without_the_permission_is_refused_with_the_success_false_401() {
        // Same gate envelope as every other permission-guarded route: the
        // `parse_and_validate` check returns before the service is fetched.
        let token = signed_token(&["oxidauth:users:read"]);
        let (app, calls) = app(false);

        let response = app
            .oneshot(post(Some(&token)))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            json_envelope(response).await,
            serde_json::json!({ "success": false })
        );
        assert_eq!(
            call_count(&calls),
            0,
            "an unpermitted jwt must not reach the code-minting service"
        );
    }

    #[tokio::test]
    async fn permitted_caller_still_receives_the_code_payload() {
        // Steps 1/3 keep the code in the body for gated callers only (support
        // desk hands it out out-of-band); the success envelope for them is
        // unchanged from pre-fix. Both the exact permission and the seeded
        // admin wildcard answer the challenge.
        for granted in [PERMISSION, "oxidauth:**:**"] {
            let token = signed_token(&[granted]);
            let (app, calls) = app(false);

            let response = app
                .oneshot(post(Some(&token)))
                .await
                .expect("should be able to serve the request");

            assert_eq!(response.status(), StatusCode::OK, "granted {granted}");
            assert_eq!(
                json_envelope(response).await,
                serde_json::json!({ "success": true, "payload": { "code": "123456" } }),
                "gated callers keep the exact pre-fix success body (granted {granted})"
            );
            assert_eq!(call_count(&calls), 1);
        }
    }

    #[tokio::test]
    async fn service_failure_blinds_to_the_generic_success() {
        // Step 1.2: any use-case failure (unknown user, no TOTP secret,
        // delete failure) logged the real error and answered the same
        // generic success — the 200-vs-400 split was an existence/enrollment
        // oracle and the old 400 embedded the raw sqlx `Display`.
        let token = signed_token(&[PERMISSION]);
        let (app, calls) = app(true);

        let response = app
            .oneshot(post(Some(&token)))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_envelope(response).await,
            serde_json::json!({ "success": true }),
            "no payload, no errors field — the database error must not reach the wire"
        );
        assert_eq!(call_count(&calls), 1);
    }

    #[tokio::test]
    async fn the_body_user_id_passes_through_to_the_service_verbatim() {
        // Slice minimum (OXA-000059 resolution revision, item 2): params
        // pass-through. The handler forwards `Json(params)` unchanged — the
        // user_id on the wire is the user_id the use case receives.
        let user_id = Uuid::new_v4();
        let token = signed_token(&[PERMISSION]);
        let (app, calls) = app(false);

        let response = app
            .oneshot(post_user(Some(&token), user_id))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            received(&calls),
            vec![user_id],
            "the body user_id must reach the service verbatim"
        );
    }
}
