pub mod health;
pub mod live;

use axum::{Router, routing::get};
use provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/healthcheck", get(health::handler))
        .route("/livecheck", get(live::handler))
        // deprecated snake_case aliases — remove after one release
        // (see README)
        .route("/health_check", get(health::handler))
        .route("/live_check", get(live::handler))
}
