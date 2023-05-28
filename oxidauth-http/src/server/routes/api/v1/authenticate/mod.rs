use axum::{response::IntoResponse, routing::post, Router};

pub fn router() -> Router {
    Router::new().route("/", post(authenticate))
}

async fn authenticate() -> impl IntoResponse {
    "ok"
}
