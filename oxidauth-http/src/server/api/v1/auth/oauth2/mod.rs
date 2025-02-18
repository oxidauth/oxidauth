// pub mod authenticate;
pub mod redirect;

use axum::{Router, routing::post};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        // .route(
        //     "/authenticate",
        //     post(authenticate::handle),
        // )
        .route(
            "/redirect",
            post(redirect::handle),
        )
}
