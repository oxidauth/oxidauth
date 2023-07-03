use axum::{response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use sqlx::PgPool;

pub mod api;

pub fn router(database: &PgPool) -> Router {
    Router::new()
        .nest("/api", api::router(database))
        .route("/__healthz", get(healthz))
}

pub async fn healthz() -> impl IntoResponse {
    Json(HealthRes { success: true })
}

#[derive(Serialize)]
pub struct HealthRes {
    success: bool,
}
