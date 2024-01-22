use axum::{extract::State, response::IntoResponse};

use crate::provider::Provider;

#[tracing::instrument(name = "authenticate_handler", skip(provider))]
pub async fn handle(State(provider): State<Provider>) -> impl IntoResponse {
    ""
}
