use axum::Router;

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
}
