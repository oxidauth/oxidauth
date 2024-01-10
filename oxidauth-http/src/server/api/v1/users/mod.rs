pub mod create_user;
pub mod find_user_by_id;

use axum::{
    routing::{get, post},
    Router,
};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/", post(create_user::handle))
        .route(
            "/:user_id",
            get(find_user_by_id::handle),
        )
}
