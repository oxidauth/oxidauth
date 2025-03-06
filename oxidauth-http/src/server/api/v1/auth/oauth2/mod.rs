pub mod callback;
pub mod redirect;

use axum::{
    Router,
    routing::{get, post},
};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/redirect", post(redirect::handle))
        .route("/callback/:client_key", get(callback::handle))
}
