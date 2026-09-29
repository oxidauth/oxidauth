use axum::{extract::State, http::StatusCode, response::IntoResponse};
use oxidauth_kernel::provider::Provider;
use oxidauth_postgres::Database;
use tracing::{error, info};

#[tracing::instrument(name = "GET /health_check", skip(provider))]
pub async fn handler(State(provider): State<Provider>) -> impl IntoResponse {
    let db = provider.fetch::<Database>();

    match db.ping().await {
        Ok(_) => {
            info!("successfully pinged database");

            StatusCode::OK
        },
        Err(err) => {
            error!(msg = "error connecting to database", err = ?err);

            StatusCode::INTERNAL_SERVER_ERROR
        },
    }
}
