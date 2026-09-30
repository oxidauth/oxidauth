pub mod accept_invitation;
pub mod create_invitation;
pub mod delete_invitation;
pub mod find_invitation;

use axum::{
    Router,
    routing::{delete, get, post},
};
use provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/", post(create_invitation::handle))
        .route("/{invitation_id}", get(find_invitation::handle))
        .route("/{invitation_id}", post(accept_invitation::handle))
        .route("/{invitation_id}", delete(delete_invitation::handle))
}
