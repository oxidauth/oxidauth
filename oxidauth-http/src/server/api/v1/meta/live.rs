use axum::{extract::State, http::StatusCode, response::IntoResponse};
use oxidauth_kernel::provider::Provider;
use oxidauth_postgres::Database;

pub async fn handler(State(provider): State<Provider>) -> impl IntoResponse {
    let db = provider.fetch::<Database>();

    match db.ping().await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
