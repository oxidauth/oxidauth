pub mod health;
pub mod live;

use axum::{routing::get, Router};
use oxidauth_kernel::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/health_check",
            get(health::handler),
        )
        .route(
            "/live_check",
            get(live::handler),
        )
}
