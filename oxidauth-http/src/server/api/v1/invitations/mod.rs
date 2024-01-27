pub mod create_invitaions;

use axum::{routing::post, Router};
use oxidauth_kernel::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new().route(
        "/",
        post(create_invitaions::handle),
    )
}
