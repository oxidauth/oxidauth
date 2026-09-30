use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use oxidauth_http::{
    Response,
    users::find_user_by_id::{FindUserByIdReq, FindUserByIdRes},
};
use oxidauth_kernel::{error::IntoOxidAuthError, users::find_user_by_id::*};
use oxidauth_permission::parse_and_validate_multiple;
use tracing::{info, warn};

use crate::{
    middleware::permission_extractor::{ExtractEntitlements, ExtractJwt},
    provider::Provider,
};

#[tracing::instrument(name = "find_user_by_id_handler", skip(provider))]
pub async fn handle(
    State(provider): State<Provider>,
    ExtractJwt(jwt): ExtractJwt,
    ExtractEntitlements(permissions): ExtractEntitlements,
    Path(params): Path<FindUserByIdReq>,
) -> impl IntoResponse {
    let mut challenges = vec!["oxidauth:users:manage".to_string()];

    if let Some(user_id) = jwt.sub {
        challenges.push(format!("oxidauth:users.{}:read", user_id));
    }

    match parse_and_validate_multiple(&challenges, &permissions) {
        Ok(true) => info!("{:?} has {:?}", jwt.sub, challenges),
        Ok(false) => {
            warn!("{:?} doesn't have {:?}", jwt.sub, challenges);

            return Response::unauthorized();
        },
        Err(err) => return Response::bad_request().error(err.to_string()),
    }

    let service = provider.fetch_unchecked::<FindUserByIdService>();

    info!("provided FindUserByIdService");

    let result = service
        .find_user_by_id(&params)
        .await;

    match result {
        Ok(user) => {
            info!(
                message = "successfully found user by id",
                user = ?user,
            );

            Response::success().payload(FindUserByIdRes { user })
        },
        Err(err) => {
            info!(
                message = "failed to find user by id",
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
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::*;

    // callback.rs pattern with the permission_extractor.rs harness (real
    // jwts, mocked public keys, real extractors), driving the REAL
    // `users::router()` at its production path. Pins the self-scope challenge
    // leg of the deviation census: `oxidauth:users.<jwt.sub>:read` is added
    // next to `oxidauth:users:manage` — the privilege shape no hurl identity
    // ever holds. (The gate's `Err` arm is unreachable here:
    // `parse_and_validate_multiple` swallows parse errors with `continue` and
    // can only ever return `Ok`, so no test can drive the 400 parse-error leg.)

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

    struct MockFindUserById {
        fail: bool,
        calls: Arc<Mutex<Vec<Uuid>>>,
    }

    #[async_trait]
    impl FindUserByIdServiceTrait for MockFindUserById {
        async fn find_user_by_id(&self, params: &FindUserById) -> Result<User, BoxedError> {
            self.calls
                .lock()
                .expect("log")
                .push(params.user_id);

            if self.fail {
                return Err("mock: user lookup failed".into());
            }

            Ok(user_fixture(params.user_id))
        }
    }

    fn user_fixture(id: Uuid) -> User {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "kind": "human",
            "status": "enabled",
            "username": "ada",
            "email": null,
            "first_name": null,
            "last_name": null,
            "profile": {},
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
        }))
        .expect("valid user fixture")
    }

    fn app(fail_service: bool) -> (Router, Arc<Mutex<Vec<Uuid>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));

        let mut provider = Provider::new();

        provider.store::<ListAllPublicKeysService>(Arc::new(MockPublicKeys));
        provider.store::<FindUserByIdService>(Arc::new(MockFindUserById {
            fail: fail_service,
            calls: Arc::clone(&calls),
        }));

        (
            Router::new()
                .nest("/users", crate::server::api::v1::users::router())
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

    fn get(token: &str, user_id: Uuid) -> Request {
        Request::builder()
            .uri(format!("/users/{user_id}"))
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::empty())
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
    async fn manage_grant_returns_the_user_and_passes_the_path_id_through() {
        let user_id = Uuid::new_v4();
        let token = signed_token(Some(Uuid::new_v4()), &["oxidauth:users:manage"]);
        let (app, calls) = app(false);

        let response = app
            .oneshot(get(&token, user_id))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);

        let envelope = json_envelope(response).await;
        assert_eq!(envelope["success"], serde_json::Value::Bool(true));
        assert_eq!(envelope["payload"]["user"]["id"], user_id.to_string());
        assert_eq!(envelope["payload"]["user"]["username"], "ada");
        assert_eq!(
            calls
                .lock()
                .expect("log")
                .as_slice(),
            &[user_id],
            "the path uuid must reach the service verbatim"
        );
    }

    #[tokio::test]
    async fn admin_wildcard_grants_the_challenge() {
        let user_id = Uuid::new_v4();
        let token = signed_token(Some(Uuid::new_v4()), &["oxidauth:**:**"]);
        let (app, calls) = app(false);

        let response = app
            .oneshot(get(&token, user_id))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(call_count(&calls), 1);
    }

    #[tokio::test]
    async fn a_self_scoped_grant_on_own_id_grants_read_without_manage() {
        // The self-scope challenge leg: with `sub` = own id and only
        // `oxidauth:users.<own-id>:read` granted (no manage), the second
        // challenge matches and the lookup runs.
        let own_id = Uuid::new_v4();
        let granted = format!("oxidauth:users.{own_id}:read");
        let token = signed_token(Some(own_id), &[granted.as_str()]);
        let (app, calls) = app(false);

        let response = app
            .oneshot(get(&token, own_id))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_envelope(response).await["payload"]["user"]["id"],
            own_id.to_string()
        );
        assert_eq!(call_count(&calls), 1);
    }

    #[tokio::test]
    async fn a_self_scoped_grant_on_someone_elses_id_is_refused_with_the_401_envelope() {
        let other_id = Uuid::new_v4();
        let granted = format!("oxidauth:users.{other_id}:read");
        let token = signed_token(Some(Uuid::new_v4()), &[granted.as_str()]);
        let (app, calls) = app(false);

        let response = app
            .oneshot(get(&token, other_id))
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
            "a foreign self-scope grant must not reach the lookup"
        );
    }

    #[tokio::test]
    async fn lookup_failure_maps_to_the_canonical_400_error_envelope() {
        let token = signed_token(Some(Uuid::new_v4()), &["oxidauth:users:manage"]);
        let (app, calls) = app(true);

        let response = app
            .oneshot(get(&token, Uuid::new_v4()))
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let envelope = json_envelope(response).await;
        assert_eq!(envelope["success"], serde_json::Value::Bool(false));
        assert_eq!(envelope["errors"][0]["name"], "BoxedError");
        assert_eq!(envelope["errors"][0]["display"], "mock: user lookup failed");
        assert_eq!(call_count(&calls), 1);
    }

    #[tokio::test]
    async fn anonymous_get_is_refused_with_401_before_the_service_runs() {
        let user_id = Uuid::new_v4();
        let (app, calls) = app(false);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/users/{user_id}"))
                    .body(Body::empty())
                    .expect("should be able to build a request"),
            )
            .await
            .expect("should be able to serve the request");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(call_count(&calls), 0);
    }
}
