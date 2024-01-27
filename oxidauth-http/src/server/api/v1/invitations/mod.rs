pub mod create_invitaions;
pub mod find_invitation;

use axum::{
    routing::{get, post},
    Router,
};
use oxidauth_kernel::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/",
            post(create_invitaions::handle),
        )
        .route(
            "/:invitation_id",
            get(find_invitation::handle),
        )
}
