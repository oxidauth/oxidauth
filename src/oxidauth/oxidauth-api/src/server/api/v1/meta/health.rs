use axum::{extract::State, response::IntoResponse};
use oxidauth_http::{__meta::healthcheck::HealthcheckRes, Response};
use oxidauth_postgres::{Database, PingTrait};
use provider::Provider;
use tracing::{error, info};

#[tracing::instrument(name = "GET /__meta/healthcheck", skip(provider))]
pub async fn handler(State(provider): State<Provider>) -> impl IntoResponse {
    let db = provider.fetch_unchecked::<Database>();

    match db.ping().await {
        Ok(_) => {
            info!("successfully pinged database");

            Response::success().payload(HealthcheckRes {
                version: env!("CARGO_PKG_VERSION").to_string(),
                healthy: true,
            })
        },
        Err(err) => {
            error!(msg = "error connecting to database", err = ?err);

            Response::internal_error().payload(HealthcheckRes {
                version: env!("CARGO_PKG_VERSION").to_string(),
                healthy: false,
            })
        },
    }
}
