use leptos::prelude::*;
use leptos_router::components::Router;

use crate::{features::auth::components::ConfigErrorPage, navigation::Navigation, state::AppState};

#[component]
pub fn App() -> impl IntoView {
    let state = AppState::new();

    match state {
        Err(message) => view! { <ConfigErrorPage message /> }.into_any(),
        Ok(state) => {
            provide_context(state);

            // Page-load session bootstrap: the SDK verifies a persisted
            // (wasm) JWT against fresh public keys and refreshes it if
            // needed — `get_jwt_decoded` succeeds exactly when a session is
            // alive or could be renewed, and fails cleanly when it is not.
            Effect::new(move |_| {
                let client = state.client.get_untracked();

                wasm_bindgen_futures::spawn_local(async move {
                    let alive = client
                        .get_jwt_decoded()
                        .await
                        .ok();

                    state.session.set(alive);
                    state.booted.set(true);
                });
            });

            view! {
                <Router>
                    <Navigation />
                </Router>
            }
            .into_any()
        },
    }
}
