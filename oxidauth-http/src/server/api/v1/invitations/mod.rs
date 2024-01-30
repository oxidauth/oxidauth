pub mod accept_invitation;
pub mod create_invitaions;
pub mod delete_invitation;
pub mod find_invitation;

use axum::{
    routing::{delete, get, post},
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
        .route(
            "/:invitation_id",
            post(accept_invitation::handle),
        )
        .route(
            "/:invitation_id",
            delete(delete_invitation::handle),
        )
}
