use axum::{Json, extract::State, response::IntoResponse};
use oxidauth_http::{
    Response,
    totp::validate::{ValidateTOTPReq, ValidateTOTPRes},
};
use oxidauth_kernel::{
    error::IntoOxidAuthError,
    totp::validate::{ValidateTOTP, ValidateTOTPService},
};
use oxidauth_permission::parse_and_validate;
use tracing::{info, warn};

use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "validate_totp_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Json(params): Json<ValidateTOTPReq>,
) -> impl IntoResponse {
    match parse_and_validate("oxidauth:totp_code:validate", &permissions) {
        Ok(true) => info!("{:?} has {}", jwt.sub, "oxidauth:totp_code:validate"),
        Ok(false) => {
            warn!(
                "{:?} doesn't have {}",
                jwt.sub, "oxidauth:totp_code:validate"
            );

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<ValidateTOTPService>();

    let user_id = match jwt.sub {
        Some(user_id) => user_id,
        None => return Response::unauthorized(),
    };

    let validation_params = ValidateTOTP {
        user_id,
        code: params.code,
        client_key: params.client_key,
    };

    let result = service
        .validate_totp(&validation_params)
        .await;

    match result {
        Ok(response) => {
            info!(
                message = "successfully authenticated with 2fa",
                response = ?response,
            );

            Response::success().payload(ValidateTOTPRes {
                jwt: response.jwt,
                refresh_token: response.refresh_token,
            })
        },
        Err(err) => {
            info!(
                message = "failed to authenticate",
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
        totp::{TOTPValidationRes, validate::ValidateTOTPServiceTrait},
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::*;

    // callback.rs pattern with the permission_extractor.rs harness: one RSA
    // keypair signs real jwts that the mocked public-key service lets the real
    // `ExtractJwt`/`ExtractEntitlements` extractors verify, so the gate and the
    // `jwt.sub` requirement are exercised exactly as production runs them. The
    // REAL `totp::router()` is mounted at its production path.

    const PERMISSION: &str = "oxidauth:totp_code:validate";

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

    #[derive(Clone, Debug, PartialEq)]
    struct Received {
        user_id: Uuid,
        code: String,
        client_key: Uuid,
    }

    struct MockValidateTotp {
        fail: bool,
        refresh_token: Uuid,
        calls: Arc<Mutex<Vec<Received>>>,
    }

    #[async_trait]
    impl ValidateTOTPServiceTrait for MockValidateTotp {
        async fn validate_totp(
            &self,
            params: &ValidateTOTP,
        ) -> Result<TOTPValidationRes, BoxedError> {
            self.calls
                .lock()
                .expect("log")
                .push(Received {
                    user_id: params.user_id,
                    code: params.code.clone(),
                    client_key: params.client_key,
                });

            if self.fail {
                return Err("mock: totp code mismatch".into());
            }

            Ok(TOTPValidationRes {
                jwt: "validated-jwt".to_string(),
                refresh_token: self.refresh_token,
            })
        }
    }

    fn app(fail_service: bool) -> (Router, Uuid, Arc<Mutex<Vec<Received>>>) {
        let refresh_token = Uuid::new_v4();
        let calls = Arc::new(Mutex::new(Vec::new()));

        let mut provider = Provider::new();

        provider.store::<ListAllPublicKeysService>(Arc::new(MockPublicKeys));
        provider.store::<ValidateTOTPService>(Arc::new(MockValidateTotp {
            fail: fail_service,
            refresh_token,
            calls: Arc::clone(&calls),
        }));

        (
            Router::new()
                .nest("/totp", crate::server::api::v1::totp::router())
                .with_state(provider),
            refresh_token,
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

    fn signed_token(sub: Option<Uuid>, permissions: &[&str]) -> String {
        let permissions: Vec<String> = permissions
            .iter()
            .map(ToString::to_string)
            .collect();

        let mut builder = Jwt::builder().with_expires_in(std::time::Duration::from_secs(600));

        if let Some(sub) = sub {
            builder = builder.with_subject(sub);
        }

        builder
            .with_entitlements(EntitlementsEncoding::Txt, &permissions)
            .build()
            .expect("should be able to build a jwt")
            .encode(&KEYPAIR.private)
            .expect("should be able to sign the jwt")
    }

    fn post(token: Option<&str>, code: &str, client_key: Uuid) -> Request {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/totp/validate")
            .header(CONTENT_TYPE, "application/json");

        if let Some(token) = token {
            builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
        }

        builder
            .body(Body::from(format!(
                r#"{{"code":"{code}","client_key":"{client_key}"}}"#
            )))
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
    async fn permitted_validation_answers_200_with_jwt_and_refresh_token() {
        let sub = Uuid::new_v4();
        let client_key = Uuid::new_v4();
        let token = signed_token(Some(sub), &[PERMISSION]);
        let (app, refresh_token, calls) = app(false);

        let response = app
            .oneshot(post(Some(&token), "123456", client_key))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_envelope(response).await,
            serde_json::json!({
                "success": true,
                "payload": { "jwt": "validated-jwt", "refresh_token": refresh_token },
            })
        );
        assert_eq!(call_count(&calls), 1);
    }

    #[tokio::test]
    async fn jwt_subject_and_body_fields_pass_through_as_validate_params() {
        let sub = Uuid::new_v4();
        let client_key = Uuid::new_v4();
        let token = signed_token(Some(sub), &[PERMISSION]);
        let (app, _refresh_token, calls) = app(false);

        let response = app
            .oneshot(post(Some(&token), "654321", client_key))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            received(&calls),
            vec![Received {
                user_id: sub,
                code: "654321".to_string(),
                client_key,
            }],
            "user_id must come from jwt.sub, code/client_key from the body"
        );
    }

    #[tokio::test]
    async fn missing_subject_is_refused_with_401_before_the_service_runs() {
        // The deviation pin (OXA-000059 census): `jwt.sub` is required here,
        // outside the canonical template — a token that passes the gate but
        // carries no subject gets the bodyless-401 envelope without ever
        // reaching the use case.
        let token = signed_token(None, &[PERMISSION]);
        let (app, _refresh_token, calls) = app(false);

        let response = app
            .oneshot(post(Some(&token), "123456", Uuid::new_v4()))
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
            "a subjectless token must not reach the validation service"
        );
    }

    #[tokio::test]
    async fn validation_failure_maps_to_the_canonical_400_error_envelope() {
        let token = signed_token(Some(Uuid::new_v4()), &[PERMISSION]);
        let (app, _refresh_token, calls) = app(true);

        let response = app
            .oneshot(post(Some(&token), "000000", Uuid::new_v4()))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let envelope = json_envelope(response).await;
        assert_eq!(envelope["success"], serde_json::Value::Bool(false));
        assert_eq!(envelope["errors"][0]["name"], "BoxedError");
        assert_eq!(envelope["errors"][0]["display"], "mock: totp code mismatch");
        assert_eq!(call_count(&calls), 1);
    }

    #[tokio::test]
    async fn valid_jwt_without_the_permission_is_refused_with_the_success_false_401() {
        let token = signed_token(Some(Uuid::new_v4()), &["oxidauth:users:read"]);
        let (app, _refresh_token, calls) = app(false);

        let response = app
            .oneshot(post(Some(&token), "123456", Uuid::new_v4()))
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
    async fn a_malformed_granted_permission_maps_to_the_400_parse_error() {
        // The gate's `Err` arm: `parse_and_validate` parses every granted
        // permission, and one unparseable entry (three colons) short-circuits
        // into `Response::bad_request().error(err.to_string())`.
        let token = signed_token(Some(Uuid::new_v4()), &["too:many:colons:here"]);
        let (app, _refresh_token, calls) = app(false);

        let response = app
            .oneshot(post(Some(&token), "123456", Uuid::new_v4()))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            json_envelope(response).await,
            serde_json::json!({ "success": false, "errors": ["invalid permission"] })
        );
        assert_eq!(call_count(&calls), 0);
    }

    #[tokio::test]
    async fn anonymous_post_is_refused_with_401_before_the_service_runs() {
        let (app, _refresh_token, calls) = app(false);

        let response = app
            .oneshot(post(None, "123456", Uuid::new_v4()))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(call_count(&calls), 0);
    }
}
