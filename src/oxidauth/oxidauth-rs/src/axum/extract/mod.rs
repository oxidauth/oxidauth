pub use axum::extract::FromRef;
use axum::{
    RequestPartsExt,
    extract::FromRequestParts,
    http::{self, request::Parts},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use oxidauth_http::public_keys::list_all_public_keys::ListAllPublicKeysRes;
use oxidauth_kernel::jwt::Jwt;
use tracing::error;
use uuid::Uuid;

use crate::{OxidAuthClient, client::public_keys::list_all_public_keys::ListAllPublicKeysTrait};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtractJwt(pub Jwt);

impl<S> FromRequestParts<S> for ExtractJwt
where
    OxidAuthClient: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = http::StatusCode;

    #[tracing::instrument(name = "oxidauth extract jwt", level = "trace", skip_all, ret, err)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let client = OxidAuthClient::from_ref(state);

        #[cfg(feature = "mock")]
        if let Some(jwt) = client.mock_jwt {
            return Ok(ExtractJwt(jwt));
        }

        decode_jwt(parts, client).await
    }
}

async fn decode_jwt(
    parts: &mut Parts,
    client: OxidAuthClient,
) -> Result<ExtractJwt, http::StatusCode> {
    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|err| {
            error!(msg = "error getting authorization header", err = ?err);

            http::StatusCode::UNAUTHORIZED
        })?;

    let ListAllPublicKeysRes { public_keys } = client
        .list_all_public_keys()
        .await
        .map_err(|err| {
            error!(msg = "error getting public keys", err = ?err);

            http::StatusCode::UNAUTHORIZED
        })?;

    let jwt = Jwt::decode_with_public_keys(bearer.token(), &public_keys).map_err(|err| {
        error!(msg = "error decoding public keys", err = ?err);

        http::StatusCode::UNAUTHORIZED
    })?;

    Ok(ExtractJwt(jwt))
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtractEntitlements(pub Vec<String>);

impl<S> FromRequestParts<S> for ExtractEntitlements
where
    OxidAuthClient: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = http::StatusCode;

    #[tracing::instrument(
        name = "oxidauth extract entitlements",
        level = "trace",
        skip_all,
        ret,
        err
    )]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ExtractJwt(jwt) = ExtractJwt::from_request_parts(parts, state).await?;

        let permissions = jwt
            .entitlements
            .and_then(|entitlements| entitlements.as_vec())
            .unwrap_or_default();

        Ok(ExtractEntitlements(permissions))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtractUserId(pub Uuid);

impl<S> FromRequestParts<S> for ExtractUserId
where
    OxidAuthClient: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = http::StatusCode;

    #[tracing::instrument(
        name = "oxidauth extract entitlements",
        level = "trace",
        skip_all,
        ret,
        err
    )]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ExtractJwt(jwt) = ExtractJwt::from_request_parts(parts, state).await?;

        let Some(user_id) = jwt.sub else {
            error!("error getting sub from jwt");

            return Err(http::StatusCode::UNAUTHORIZED);
        };

        Ok(ExtractUserId(user_id))
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::sync::LazyLock;

    use axum::{
        extract::{FromRef, FromRequestParts},
        http::{Request, StatusCode, request::Parts},
    };
    use oxidauth_kernel::{
        jwt::{Entitlements, Jwt},
        rsa::KeyPair,
    };
    use serde_json::json;
    use uuid::Uuid;
    use wiremock::{
        Mock,
        MockServer,
        ResponseTemplate,
        matchers::{method, path},
    };

    use super::*;

    // one RSA pair for the whole suite: generation is the expensive part
    static KP: LazyLock<KeyPair> = LazyLock::new(|| KeyPair::new().unwrap());
    fn kp() -> &'static KeyPair {
        &KP
    }

    #[derive(Clone)]
    struct AppState {
        client: OxidAuthClient,
    }

    impl FromRef<AppState> for OxidAuthClient {
        fn from_ref(state: &AppState) -> Self {
            state.client.clone()
        }
    }

    fn parts_with_auth(header: Option<&str>) -> Parts {
        let mut builder = Request::builder().uri("/");
        if let Some(header) = header {
            builder = builder.header("authorization", header);
        }
        builder
            .body(())
            .unwrap()
            .into_parts()
            .0
    }

    async fn state_with_keys(body: String) -> (AppState, MockServer) {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/public_keys"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&server)
            .await;

        let base_url = url::Url::parse(&server.uri()).unwrap();
        let client = OxidAuthClient::new(&base_url, Uuid::new_v4()).unwrap();
        (AppState { client }, server)
    }

    fn pem_keys_body() -> String {
        serde_json::to_string(&json!({
            "success": true,
            "payload": { "public_keys": [{
                "id": Uuid::new_v4().to_string(),
                // decode_with_public_keys feeds the stored string verbatim into
                // from_rsa_pem, so the mock serves raw PEM (pinned by E2 tests
                // in client/mod.rs)
                "public_key": std::str::from_utf8(&kp().public).unwrap(),
                "created_at": "2026-09-29T00:00:00Z",
                "updated_at": "2026-09-29T00:00:00Z",
            }]},
        }))
        .unwrap()
    }

    async fn state_with_real_keys() -> (AppState, MockServer) {
        state_with_keys(pem_keys_body()).await
    }

    fn sign(claims: &Jwt) -> String {
        claims
            .encode(&kp().private)
            .unwrap()
    }

    fn valid_claims(sub: Uuid) -> Jwt {
        Jwt::builder()
            .with_subject(sub)
            .with_expires_at((chrono::Utc::now().timestamp() + 3600) as usize)
            .build()
            .unwrap()
    }

    // ---------------------------------------------------------- ExtractJwt

    #[tokio::test]
    async fn missing_authorization_header_is_401_without_touching_the_network() {
        let (state, server) = state_with_real_keys().await;
        let mut parts = parts_with_auth(None);

        let err = ExtractJwt::from_request_parts(&mut parts, &state)
            .await
            .expect_err("headerless request must be rejected");

        assert_eq!(
            err,
            StatusCode::UNAUTHORIZED,
            "rejection is a bare status code"
        );
        assert!(
            server
                .received_requests()
                .await
                .unwrap()
                .is_empty(),
            "header extraction must fail before any public-keys fetch"
        );
    }

    #[tokio::test]
    async fn non_bearer_or_malformed_authorization_header_is_401() {
        let (state, server) = state_with_real_keys().await;

        for header in ["Basic dXNlcjpwYXNz", "Bearer", "random-token"] {
            let mut parts = parts_with_auth(Some(header));
            let err = ExtractJwt::from_request_parts(&mut parts, &state)
                .await
                .expect_err("{header} is not a well-formed bearer");
            assert_eq!(err, StatusCode::UNAUTHORIZED, "case: {header}");
        }
        assert!(
            server
                .received_requests()
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn valid_bearer_with_server_keys_extracts_the_claims() {
        let (state, server) = state_with_real_keys().await;
        let sub = Uuid::new_v4();
        let token = sign(&valid_claims(sub));
        let mut parts = parts_with_auth(Some(&format!("Bearer {token}")));

        let ExtractJwt(jwt) = ExtractJwt::from_request_parts(&mut parts, &state)
            .await
            .expect("token signed by the served keypair must verify");

        assert_eq!(jwt.sub, Some(sub));
        assert_eq!(
            server
                .received_requests()
                .await
                .unwrap()
                .iter()
                .filter(|r| r.url.path() == "/api/v1/public_keys")
                .count(),
            1,
            "keys are fetched exactly once per extraction"
        );
    }

    #[tokio::test]
    async fn unverifiable_bearer_is_401() {
        let (state, _server) = state_with_real_keys().await;

        // garbage token...
        let mut parts = parts_with_auth(Some("Bearer not.a.jwt"));
        assert_eq!(
            ExtractJwt::from_request_parts(&mut parts, &state)
                .await
                .unwrap_err(),
            StatusCode::UNAUTHORIZED
        );

        // ...and a well-formed token signed by a foreign key
        let foreign = KeyPair::new().unwrap();
        let forged = valid_claims(Uuid::new_v4())
            .encode(&foreign.private)
            .unwrap();
        let mut parts = parts_with_auth(Some(&format!("Bearer {forged}")));
        assert_eq!(
            ExtractJwt::from_request_parts(&mut parts, &state)
                .await
                .unwrap_err(),
            StatusCode::UNAUTHORIZED,
            "a token the served keys cannot verify must be rejected"
        );
    }

    #[tokio::test]
    async fn public_keys_fetch_failure_is_401() {
        // 200 OK but not the keys envelope: list_all_public_keys fails to
        // deserialize, decode_jwt maps it to UNAUTHORIZED
        let (state, _server) = state_with_keys("<html>".to_string()).await;
        let token = sign(&valid_claims(Uuid::new_v4()));
        let mut parts = parts_with_auth(Some(&format!("Bearer {token}")));

        assert_eq!(
            ExtractJwt::from_request_parts(&mut parts, &state)
                .await
                .unwrap_err(),
            StatusCode::UNAUTHORIZED
        );
    }

    // -------------------------------------------------- ExtractEntitlements

    #[tokio::test]
    async fn entitlements_are_sliced_from_the_txt_claim() {
        let (state, _server) = state_with_real_keys().await;
        let mut claims = valid_claims(Uuid::new_v4());
        claims.entitlements = Some(Entitlements::Txt(
            "oxidauth:users:read oxidauth:roles:*".to_string(),
        ));
        let token = sign(&claims);
        let mut parts = parts_with_auth(Some(&format!("Bearer {token}")));

        let ExtractEntitlements(entitlements) =
            ExtractEntitlements::from_request_parts(&mut parts, &state)
                .await
                .expect("txt entitlements must decode");

        assert_eq!(
            entitlements,
            vec![
                "oxidauth:users:read".to_string(),
                "oxidauth:roles:*".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn missing_entitlements_claim_yields_empty_vec_not_401() {
        let (state, _server) = state_with_real_keys().await;
        let token = sign(&valid_claims(Uuid::new_v4()));
        let mut parts = parts_with_auth(Some(&format!("Bearer {token}")));

        let ExtractEntitlements(entitlements) =
            ExtractEntitlements::from_request_parts(&mut parts, &state)
                .await
                .expect("claim absence is not a rejection (unwrap_or_default)");

        assert!(entitlements.is_empty());
    }

    // -------------------------------------------------------- ExtractUserId

    #[tokio::test]
    async fn user_id_is_the_jwt_subject() {
        let (state, _server) = state_with_real_keys().await;
        let sub = Uuid::new_v4();
        let token = sign(&valid_claims(sub));
        let mut parts = parts_with_auth(Some(&format!("Bearer {token}")));

        assert_eq!(
            ExtractUserId::from_request_parts(&mut parts, &state)
                .await
                .unwrap(),
            ExtractUserId(sub)
        );
    }

    #[tokio::test]
    async fn subjectless_jwt_is_401_for_user_id() {
        let (state, _server) = state_with_real_keys().await;
        let mut claims = valid_claims(Uuid::new_v4());
        claims.sub = None;
        let token = sign(&claims);
        let mut parts = parts_with_auth(Some(&format!("Bearer {token}")));

        assert_eq!(
            ExtractUserId::from_request_parts(&mut parts, &state)
                .await
                .unwrap_err(),
            StatusCode::UNAUTHORIZED,
            "ExtractJwt accepts a subjectless token, only ExtractUserId rejects it"
        );
    }
}
