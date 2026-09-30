use axum::{
    RequestPartsExt,
    extract::{FromRef, FromRequestParts},
    http::{self, request::Parts},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use oxidauth_kernel::{
    jwt::Jwt,
    public_keys::list_all_public_keys::{ListAllPublicKeys, ListAllPublicKeysService},
};

use crate::provider::Provider;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtractJwt(pub Jwt);

impl<S> FromRequestParts<S> for ExtractJwt
where
    Provider: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = http::StatusCode;

    #[tracing::instrument(name = "oxidauth extract jwt", level = "trace", skip_all, ret, err)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

        let provider = Provider::from_ref(state);

        let list_all_public_keys_service = provider.fetch_unchecked::<ListAllPublicKeysService>();

        let public_keys = list_all_public_keys_service
            .list_all_public_keys(&ListAllPublicKeys)
            .await
            .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

        let jwt = Jwt::decode_with_public_keys(bearer.token(), &public_keys)
            .map_err(|_| http::StatusCode::UNAUTHORIZED)?;

        Ok(ExtractJwt(jwt))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtractEntitlements(pub Vec<String>);

impl FromRequestParts<Provider> for ExtractEntitlements {
    type Rejection = http::StatusCode;

    #[tracing::instrument(
        name = "oxidauth extract entitlements",
        level = "trace",
        skip_all,
        ret,
        err
    )]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &Provider,
    ) -> Result<Self, Self::Rejection> {
        let ExtractJwt(jwt) = ExtractJwt::from_request_parts(parts, state).await?;

        let permissions = jwt
            .entitlements
            .and_then(|entitlements| entitlements.as_vec())
            .unwrap_or_default();

        Ok(ExtractEntitlements(permissions))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, LazyLock},
        time::Duration,
    };

    use async_trait::async_trait;
    use axum::{
        Router,
        body::Body,
        extract::{FromRequestParts, Request},
        http::{StatusCode, header::AUTHORIZATION},
        routing::get,
    };
    use http_body_util::BodyExt;
    use oxidauth_kernel::{
        error::BoxedError,
        jwt::{EntitlementsEncoding, Jwt},
        public_keys::{PublicKey, list_all_public_keys::*},
        rsa::KeyPair,
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::*;

    // One RSA keypair shared across the suite; `KeyPair::new()` is a public
    // kernel API (PKCS#8 private PEM + SPKI public PEM), so the tests sign and
    // verify exactly like production without adding an `rsa` dev-dep here.
    static KEYPAIR: LazyLock<KeyPair> =
        LazyLock::new(|| KeyPair::new().expect("should be able to generate a test keypair"));

    struct MockListAllPublicKeys {
        public_keys: Vec<String>,
        fail: bool,
    }

    #[async_trait]
    impl ListAllPublicKeysServiceTrait for MockListAllPublicKeys {
        async fn list_all_public_keys(
            &self,
            _params: &ListAllPublicKeys,
        ) -> Result<Vec<PublicKey>, BoxedError> {
            if self.fail {
                return Err("mock: public-key service unavailable".into());
            }

            Ok(self
                .public_keys
                .iter()
                .map(|pem| {
                    PublicKey {
                        id: Uuid::nil(),
                        public_key: pem.clone(),
                        created_at: serde_json::from_str("\"2026-01-01T00:00:00Z\"")
                            .expect("should be able to parse a fixed timestamp"),
                        updated_at: serde_json::from_str("\"2026-01-01T00:00:00Z\"")
                            .expect("should be able to parse a fixed timestamp"),
                    }
                })
                .collect())
        }
    }

    fn provider_with(public_keys: Vec<String>, fail: bool) -> Provider {
        let mut provider = Provider::new();

        provider.store::<ListAllPublicKeysService>(Arc::new(MockListAllPublicKeys {
            public_keys,
            fail,
        }));

        provider
    }

    fn provider_with_matching_key() -> Provider {
        provider_with(
            vec![String::from_utf8(KEYPAIR.public.clone()).expect("pem is utf8")],
            false,
        )
    }

    fn signed_token(sub: Uuid, permissions: Option<(&EntitlementsEncoding, &[&str])>) -> String {
        let builder = Jwt::builder()
            .with_subject(sub)
            .with_expires_in(Duration::from_secs(600));

        let jwt = match permissions {
            Some((encoding, perms)) => {
                let perms: Vec<String> = perms
                    .iter()
                    .map(ToString::to_string)
                    .collect();

                builder
                    .with_entitlements(encoding.clone(), &perms)
                    .build()
                    .expect("should be able to build a jwt")
            },
            None => {
                builder
                    .build()
                    .expect("should be able to build a jwt")
            },
        };

        jwt.encode(&KEYPAIR.private)
            .expect("should be able to sign the jwt")
    }

    fn bearer(token: &str) -> String {
        format!("Bearer {token}")
    }

    fn parts_with_authorization(value: Option<&str>) -> Parts {
        let mut request = Request::new(Body::empty());

        if let Some(value) = value {
            request.headers_mut().insert(
                AUTHORIZATION,
                value
                    .parse()
                    .expect("valid header value"),
            );
        }

        request.into_parts().0
    }

    async fn collect_body(response: axum::response::Response) -> Vec<u8> {
        BodyExt::collect(response.into_body())
            .await
            .expect("should be able to read the response body")
            .to_bytes()
            .to_vec()
    }

    #[tokio::test]
    async fn extract_jwt_rejects_missing_authorization_header_with_401() {
        let mut parts = parts_with_authorization(None);
        let provider = provider_with_matching_key();

        let result = ExtractJwt::from_request_parts(&mut parts, &provider).await;

        assert_eq!(result.err(), Some(StatusCode::UNAUTHORIZED));
    }

    #[tokio::test]
    async fn extract_jwt_rejects_non_bearer_authorization_with_401() {
        let mut parts = parts_with_authorization(Some("Basic dXNlcjpwYXNz"));
        let provider = provider_with_matching_key();

        let result = ExtractJwt::from_request_parts(&mut parts, &provider).await;

        assert_eq!(result.err(), Some(StatusCode::UNAUTHORIZED));
    }

    #[tokio::test]
    async fn extract_jwt_rejects_undecodable_token_with_401() {
        let token = signed_token(Uuid::new_v4(), None);

        let mut parts = parts_with_authorization(Some(&bearer("obviously.not.a.jwt")));
        let provider = provider_with_matching_key();

        let result = ExtractJwt::from_request_parts(&mut parts, &provider).await;

        assert_eq!(result.err(), Some(StatusCode::UNAUTHORIZED));

        // sanity: the token we did sign verifies against the same provider,
        // so the rejection above is the malformed token, not the fixture setup
        let mut parts = parts_with_authorization(Some(&bearer(&token)));

        assert!(
            ExtractJwt::from_request_parts(&mut parts, &provider)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn extract_jwt_rejects_signed_token_when_no_public_keys_exist_with_401() {
        let token = signed_token(Uuid::new_v4(), None);

        let mut parts = parts_with_authorization(Some(&bearer(&token)));
        let provider = provider_with(vec![], false);

        let result = ExtractJwt::from_request_parts(&mut parts, &provider).await;

        assert_eq!(result.err(), Some(StatusCode::UNAUTHORIZED));
    }

    #[tokio::test]
    async fn extract_jwt_rejects_when_public_key_service_fails_with_401() {
        let token = signed_token(Uuid::new_v4(), None);

        let mut parts = parts_with_authorization(Some(&bearer(&token)));
        let provider = provider_with(vec![], true);

        let result = ExtractJwt::from_request_parts(&mut parts, &provider).await;

        assert_eq!(result.err(), Some(StatusCode::UNAUTHORIZED));
    }

    #[tokio::test]
    async fn extract_jwt_yields_claims_for_valid_bearer() {
        let sub = Uuid::new_v4();
        let token = signed_token(sub, None);

        let mut parts = parts_with_authorization(Some(&bearer(&token)));
        let provider = provider_with_matching_key();

        let result = ExtractJwt::from_request_parts(&mut parts, &provider).await;

        let ExtractJwt(jwt) = result.expect("should accept a signed bearer token");

        assert_eq!(jwt.sub, Some(sub));
    }

    #[tokio::test]
    async fn extract_entitlements_rejects_missing_authorization_header_with_401() {
        let mut parts = parts_with_authorization(None);
        let provider = provider_with_matching_key();

        let result = ExtractEntitlements::from_request_parts(&mut parts, &provider).await;

        assert_eq!(result.err(), Some(StatusCode::UNAUTHORIZED));
    }

    #[tokio::test]
    async fn extract_entitlements_yields_txt_permissions() {
        let token = signed_token(
            Uuid::new_v4(),
            Some((
                &EntitlementsEncoding::Txt,
                &["oxidauth:users:read", "**:**:**"],
            )),
        );

        let mut parts = parts_with_authorization(Some(&bearer(&token)));
        let provider = provider_with_matching_key();

        let result = ExtractEntitlements::from_request_parts(&mut parts, &provider).await;

        let ExtractEntitlements(permissions) =
            result.expect("should yield entitlements from a txt claim");

        assert_eq!(
            permissions,
            vec!["oxidauth:users:read".to_string(), "**:**:**".to_string()]
        );
    }

    #[tokio::test]
    async fn extract_entitlements_yields_gz_permissions() {
        let sub = Uuid::new_v4();
        let perms: Vec<String> = ["oxidauth:roles:read", "oxidauth:roles:write"]
            .iter()
            .map(ToString::to_string)
            .collect();

        let jwt = Jwt::builder()
            .with_subject(sub)
            .with_expires_in(Duration::from_secs(600))
            .with_entitlements(EntitlementsEncoding::Gz, &perms)
            .build()
            .expect("should be able to build a gz-entitlements jwt");

        let token = jwt
            .encode(&KEYPAIR.private)
            .expect("should be able to sign");

        let mut parts = parts_with_authorization(Some(&bearer(&token)));
        let provider = provider_with_matching_key();

        let result = ExtractEntitlements::from_request_parts(&mut parts, &provider).await;

        let ExtractEntitlements(permissions) =
            result.expect("should yield entitlements from a gz claim");

        assert_eq!(
            permissions,
            vec![
                "oxidauth:roles:read".to_string(),
                "oxidauth:roles:write".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn extract_entitlements_yields_empty_list_when_jwt_has_no_entitlements() {
        // Pinch-point for the plan-11 contract wording: the extractor itself
        // NEVER rejects a permission-less jwt — it resolves to Ok(vec![]) and
        // the 401 `{"success":false}` envelope is minted by the handlers'
        // `parse_and_validate` gate (`Response::unauthorized()`), not here.
        // See `valid_jwt_without_permission_returns_success_false_envelope_401`.
        let token = signed_token(Uuid::new_v4(), None);

        let mut parts = parts_with_authorization(Some(&bearer(&token)));
        let provider = provider_with_matching_key();

        let result = ExtractEntitlements::from_request_parts(&mut parts, &provider).await;

        assert_eq!(result, Ok(ExtractEntitlements(vec![])));
    }

    #[tokio::test]
    async fn extractor_rejection_over_a_route_is_a_bodyless_401() {
        async fn guard(_jwt: ExtractJwt) -> &'static str {
            "unreachable without a valid jwt"
        }

        // Pins the SERVICE-FAILURE leg: a well-formed Bearer header carrying a
        // genuinely signed token gets past header extraction, so this 401 can
        // only come from the failing public-key service. (The missing-header
        // leg is pinned at extractor level by
        // extract_jwt_rejects_missing_authorization_header_with_401.)
        let token = signed_token(Uuid::new_v4(), None);

        let app = Router::new()
            .route("/protected", get(guard))
            .with_state(provider_with(vec![], true));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(AUTHORIZATION, bearer(&token))
                    .body(Body::empty())
                    .expect("should be able to build a request"),
            )
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            collect_body(response).await,
            Vec::<u8>::new(),
            "jwt-extractor rejection must stay bodyless (plan-11 pinned)"
        );
    }

    #[tokio::test]
    async fn valid_jwt_without_permission_returns_success_false_envelope_401() {
        // The other half of the plan-11 contract, end to end on a real route:
        // a VALID jwt whose entitlements lack `oxidauth:authorities:manage`
        // must get the 401 `{"success":false}` JSON envelope (the handler's
        // gate returns before any service is fetched, so the mocked
        // public-key service is the only provider dependency).
        let token = signed_token(
            Uuid::new_v4(),
            Some((&EntitlementsEncoding::Txt, &["oxidauth:users:read"])),
        );

        let app =
            crate::server::api::v1::authorities::router().with_state(provider_with_matching_key());

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/")
                    .header(AUTHORIZATION, bearer(&token))
                    .body(Body::empty())
                    .expect("should be able to build a request"),
            )
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = collect_body(response).await;

        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body)
                .expect("rejection body must be the JSON envelope"),
            serde_json::json!({ "success": false })
        );
    }
}
