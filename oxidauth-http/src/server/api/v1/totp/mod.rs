use axum::{
    routing::{get, post},
    Router,
};

use crate::provider::Provider;

pub mod generate;
pub mod validate;

pub fn router() -> Router<Provider> {
    Router::new().route(
        "/generate",
        get(validate::handle),
    );
    Router::new().route(
        "/validate",
        post(validate::handle),
    )
}
