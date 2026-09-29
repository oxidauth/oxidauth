use axum::{extract::State, http::StatusCode, response::IntoResponse};
use oxidauth_postgres::Database;
use provider::Provider;

pub async fn handler(State(provider): State<Provider>) -> impl IntoResponse {
    let db = provider.fetch_unchecked::<Database>();

    match db.ping().await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
