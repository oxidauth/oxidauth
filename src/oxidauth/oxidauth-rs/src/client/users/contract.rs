//! Shared table-driven contract harness for the oxidauth-rs endpoint wrappers (audit row E5).
//!
//! Every `src/client/**` wrapper test rides one of the two runners below. Per wrapper the
//! runners assert the full request/response contract against a wiremock route:
//!
//! 1. **route + verb + bearer**: the wrapper hits exactly `<base_url><route>` with the expected
//!    HTTP verb and an `Authorization: Bearer <jwt>` header (auth state is injected directly so the
//!    test isolates the wrapper from the E1 state machine);
//! 2. **payload decode**: a `success:true` envelope carrying a canned payload decodes into the
//!    wrapper's typed result and re-serializes back to the *exact* canned shape;
//! 3. **error envelope**: a `success:false` envelope surfaces as the underlying
//!    `ClientErrorKind::APIResponseError` (downcastable out of the wrapper's `BoxedError`) with the
//!    server-reported errors chained as `source`;
//! 4. **empty payload**: a `success:true` envelope without payload maps to
//!    `ClientErrorKind::EmptyPayload(resource, method)` — this also pins each wrapper's
//!    `RESOURCE`/`METHOD` labels, which leak verbatim into the error copy callers see.
//!
//! `contract_raw` is the variant for the three wrappers that return the raw
//! `Response<T>` without `handle_response`; their error leg instead *pins the absence*
//! of error mapping (a success:false envelope passes through as `Ok`).
//!
//! Lives under `client::users` (a descendant of `client`, so it may poke `Client`'s
//! private auth state) but is `pub(crate)` and imported by every domain's test module.

use std::error::Error as _;

use chrono::{TimeZone, Utc};
use oxidauth_http::Response;
use oxidauth_kernel::{error::BoxedError, jwt::Jwt};
use reqwest::header::HeaderMap;
use serde::Serialize;
use serde_json::{Value, json};
use url::Url;
use uuid::Uuid;
use wiremock::{
    Mock,
    MockServer,
    ResponseTemplate,
    matchers::{method, path},
};

use crate::client::{Client, ClientError, ClientErrorKind};

pub(crate) const TEST_JWT: &str = "test.jwt.token";

/// A client whose auth state is pre-seeded with a valid (unexpired) jwt + bearer, so
/// `authenticate_if_needed` short-circuits to `AuthState::Valid` and the wrapper's own
/// HTTP call is the only traffic the mock server sees.
pub(crate) async fn authed(server: &MockServer) -> Client {
    let base_url = Url::parse(&server.uri()).unwrap();
    let client = Client::new(&base_url, Uuid::new_v4()).unwrap();

    let jwt = Jwt::builder()
        .with_subject(Uuid::new_v4())
        .with_expires_at((Utc::now().timestamp() + 3600) as usize)
        .build()
        .unwrap();

    let bearer = format!("Bearer {TEST_JWT}")
        .parse()
        .unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", bearer);

    let mut state = client.state.write().await;
    state.jwt = Some(jwt);
    state.raw_jwt = Some(TEST_JWT.to_string());
    state.client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    drop(state);

    client
}

async fn mount(server: &MockServer, verb: &str, route: &str, status: u16, body: Value) {
    Mock::given(method(verb))
        .and(path(route))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .mount(server)
        .await;
}

async fn assert_single_authed_hit(server: &MockServer, verb: &str, route: &str) {
    let hits: Vec<wiremock::Request> = server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|req| req.method.as_str() == verb && req.url.path() == route)
        .collect();

    assert_eq!(
        hits.len(),
        1,
        "wrapper must issue exactly one {verb} {route} (auth short-circuits)"
    );
    let bearer = format!("Bearer {TEST_JWT}");
    assert_eq!(
        hits[0]
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok()),
        Some(bearer.as_str()),
        "wrapper request must carry the bearer token"
    );
}

/// Table-driven contract runner for the 52 wrappers whose results travel as the
/// success/error envelope; the 3 auth wrappers with raw payload bodies use
/// [`contract_raw`] instead. Every wrapper file gets exactly one test.
pub(crate) async fn contract<T, C, F>(
    verb: &str,
    route: &str,
    resource_method: (&str, &str),
    canned: Value,
    call: C,
) where
    C: Fn(Client) -> F,
    F: Future<Output = Result<T, BoxedError>>,
    T: Serialize + std::fmt::Debug,
{
    // -- leg 1: success envelope -> typed payload, exact shape, one authed hit
    let server = MockServer::start().await;
    mount(
        &server,
        verb,
        route,
        200,
        json!({ "success": true, "payload": canned }),
    )
    .await;

    let res = call(authed(&server).await)
        .await
        .expect("success envelope must decode");
    assert_eq!(
        serde_json::to_value(&res).unwrap(),
        canned,
        "typed result must round-trip to the exact canned payload shape"
    );
    assert_single_authed_hit(&server, verb, route).await;

    // -- leg 2: success:false envelope -> APIResponseError, server errors chained
    let server = MockServer::start().await;
    mount(
        &server,
        verb,
        route,
        400,
        json!({ "success": false, "errors": ["boom"] }),
    )
    .await;

    let err = call(authed(&server).await)
        .await
        .expect_err("success:false envelope must surface as an error");
    let client_err = err
        .downcast_ref::<ClientError>()
        .expect("wrapper BoxedError must be the underlying ClientError");
    assert!(matches!(client_err.kind, ClientErrorKind::APIResponseError));
    assert_eq!(
        client_err
            .source()
            .unwrap()
            .to_string(),
        "\"boom\"",
        "reported errors are serialized JSON values, so the joined copy carries quotes"
    );
    assert_single_authed_hit(&server, verb, route).await;

    // -- leg 3: success:true without payload -> EmptyPayload(resource, method)
    let server = MockServer::start().await;
    mount(&server, verb, route, 200, json!({ "success": true })).await;

    let err = call(authed(&server).await)
        .await
        .expect_err("payloadless envelope must surface as an error");
    let client_err = err
        .downcast_ref::<ClientError>()
        .expect("wrapper BoxedError must be the underlying ClientError");
    let (resource, method_label) = resource_method;
    assert!(
        matches!(&client_err.kind, ClientErrorKind::EmptyPayload(r, m)
            if r.to_string() == resource && *m == method_label),
        "EmptyPayload must name resource `{resource}` method `{method_label}`, got {:?}",
        client_err.kind
    );
    assert!(
        client_err
            .to_string()
            .contains(&format!("resource {resource} method {method_label}")),
        "wrapper labels must be visible in the Display copy: {client_err}"
    );
}

/// Contract runner for the raw-`Response` wrappers (`oauth2_redirect`,
/// `username_password_{forgot,update}_password`): they never call `handle_response`,
/// so the error leg pins the *pass-through* instead of an error mapping.
pub(crate) async fn contract_raw<P, C, F>(verb: &str, route: &str, canned: Value, call: C)
where
    C: Fn(Client) -> F,
    F: Future<Output = Result<Response<P>, BoxedError>>,
    P: Serialize + std::fmt::Debug,
{
    // -- leg 1: success envelope -> Response { success: true, payload }
    let server = MockServer::start().await;
    mount(
        &server,
        verb,
        route,
        200,
        json!({ "success": true, "payload": canned }),
    )
    .await;

    let res = call(authed(&server).await)
        .await
        .expect("raw wrapper must not fail on a well-formed envelope");
    assert!(res.success);
    assert_eq!(
        serde_json::to_value(
            res.payload
                .as_ref()
                .expect("payload must be decoded")
        )
        .unwrap(),
        canned
    );
    assert_single_authed_hit(&server, verb, route).await;

    // -- leg 2: BUG(pinned) — the raw wrappers ignore success:false; the failure
    // envelope is returned as an *Ok* Response and only the caller can notice.
    let server = MockServer::start().await;
    mount(
        &server,
        verb,
        route,
        400,
        json!({ "success": false, "errors": ["boom"] }),
    )
    .await;

    let res = call(authed(&server).await)
        .await
        .expect("raw wrapper passes success:false through as Ok (pinned bug)");
    assert!(!res.success);
    assert!(res.payload.is_none());
    assert_eq!(res.errors, Some(vec![json!("boom")]));
    assert_single_authed_hit(&server, verb, route).await;
}

// --------------------------------------------------------------- canned entities
//
// Shapes below are the *exact* serde output of the kernel entities under test, so the
// runners' `to_value(res) == canned` assertion is a strict bidirectional contract:
// any entity field added/renamed without a wire change flips every wrapper test.

/// Fixed instant, emitted in chrono's canonical serialization so decode -> re-encode
/// of `DateTime<Utc>` is byte-stable.
pub(crate) fn ts() -> Value {
    serde_json::to_value(
        Utc.timestamp_opt(1_759_110_000, 0)
            .single()
            .unwrap(),
    )
    .unwrap()
}

pub(crate) fn uid() -> String {
    Uuid::new_v4().to_string()
}

pub(crate) fn user() -> Value {
    json!({
        "id": uid(),
        "kind": "human",
        "status": "enabled",
        "username": "malreynolds",
        "email": null,
        "first_name": null,
        "last_name": null,
        "profile": { "nickname": "mal" },
        "created_at": ts(),
        "updated_at": ts(),
    })
}

pub(crate) fn role() -> Value {
    json!({
        "id": uid(),
        "name": "admin",
        "created_at": ts(),
        "updated_at": ts(),
    })
}

pub(crate) fn permission() -> Value {
    json!({
        "id": uid(),
        "realm": "oxidauth",
        "resource": "users",
        "action": "read",
        "created_at": ts(),
        "updated_at": ts(),
    })
}

/// `std::time::Duration` rides serde's canonical `{secs, nanos}` struct form
/// (pinned by the kernel's `nbf_offset_serde_wire_shape_is_pinned`).
pub(crate) fn secs(n: u64) -> Value {
    json!({ "secs": n, "nanos": 0 })
}

pub(crate) fn authority() -> Value {
    json!({
        "id": uid(),
        "name": "primary",
        "client_key": uid(),
        "status": "enabled",
        "strategy": "username_password",
        "settings": {
            "jwt_ttl": secs(86_400),
            "jwt_nbf_offset": { "enabled": secs(10) },
            "refresh_token_ttl": secs(2_592_000),
            "totp": "disabled",
            "entitlements_encoding": "txt",
        },
        "params": { "registration_enabled": true },
        "created_at": ts(),
        "updated_at": ts(),
    })
}

pub(crate) fn public_key() -> Value {
    json!({
        "id": uid(),
        "public_key": "-----BEGIN PUBLIC KEY-----canned-----END PUBLIC KEY-----",
        "created_at": ts(),
        "updated_at": ts(),
    })
}

pub(crate) fn user_authority() -> Value {
    json!({
        "user_id": uid(),
        "authority_id": uid(),
        "user_identifier": "malreynolds",
        "params": { "password": "pass:salt:pepper" },
        "created_at": ts(),
        "updated_at": ts(),
    })
}

pub(crate) fn user_authority_with_authority() -> Value {
    json!({
        "user_authority": user_authority(),
        "authority": authority(),
    })
}

pub(crate) fn setting(key: &str) -> Value {
    json!({
        "key": key,
        "value": { "plan": "free" },
        "created_at": ts(),
        "updated_at": ts(),
    })
}

pub(crate) fn invitation() -> Value {
    json!({
        "id": uid(),
        "user_id": uid(),
        "expires_at": ts(),
        "created_at": ts(),
        "updated_at": ts(),
    })
}

pub(crate) fn user_role_grant(user_id: &str, role_id: &str) -> Value {
    json!({
        "user_id": user_id,
        "role_id": role_id,
        "created_at": ts(),
        "updated_at": ts(),
    })
}

pub(crate) fn user_role() -> Value {
    json!({
        "role": role(),
        "grant": user_role_grant(&uid(), &uid()),
    })
}

pub(crate) fn user_permission() -> Value {
    json!({
        "permission": permission(),
        "grant": {
            "user_id": uid(),
            "permission_id": uid(),
            "created_at": ts(),
            "updated_at": ts(),
        },
    })
}

pub(crate) fn role_permission() -> Value {
    json!({
        "permission": permission(),
        "grant": {
            "role_id": uid(),
            "permission_id": uid(),
            "created_at": ts(),
            "updated_at": ts(),
        },
    })
}

pub(crate) fn role_role_grant() -> Value {
    json!({
        "parent_id": uid(),
        "child_id": uid(),
        "created_at": ts(),
        "updated_at": ts(),
    })
}

pub(crate) fn role_role_grant_detail() -> Value {
    json!({
        "role": role(),
        "grant": role_role_grant(),
    })
}
