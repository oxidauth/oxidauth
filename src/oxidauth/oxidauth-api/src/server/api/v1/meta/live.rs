use axum::{extract::State, response::IntoResponse};
use oxidauth_http::{__meta::livecheck::LivecheckRes, Response};
use provider::Provider;

#[tracing::instrument(name = "GET /__meta/livecheck", skip(_provider))]
pub async fn handler(State(_provider): State<Provider>) -> impl IntoResponse {
    Response::success().payload(LivecheckRes {
        version: env!("CARGO_PKG_VERSION").to_string(),
        healthy: true,
    })
}
