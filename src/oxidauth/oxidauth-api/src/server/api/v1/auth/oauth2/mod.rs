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
        .route(
            "/callback/{client_key}",
            // GET: Google/Microsoft query callbacks. POST: Sign in with
            // Apple's `response_mode=form_post` callback (OXA-000072).
            get(callback::handle).post(callback::handle_form),
        )
}
