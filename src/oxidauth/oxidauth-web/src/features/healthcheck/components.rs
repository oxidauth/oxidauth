use http::data::LoadingState;
use leptos::prelude::*;
use oxidauth_http::{__meta::healthcheck::HealthcheckRes, Response};

use crate::state::AppState;

/// Clone/PartialEq projection of `HealthcheckRes` so it can live in a
/// signal (`ReadSignal` requires both; the wire DTO derives neither).
#[derive(Clone, PartialEq, Debug)]
struct Health {
    version: String,
    healthy: bool,
}

impl From<HealthcheckRes> for Health {
    fn from(res: HealthcheckRes) -> Self {
        Self {
            version: res.version,
            healthy: res.healthy,
        }
    }
}

/// Sidebar liveness dot: the api's own version + database health, fetched
/// once through the SDK (so the request carries the session bearer and its
/// auto-refresh applies like every other call).
#[component]
pub fn Healthcheck() -> impl IntoView {
    let state = expect_context::<AppState>();

    let (status, set_status) = signal(LoadingState::<Health>::Pending);

    Effect::new(move |_| {
        let client = state.client.get_untracked();

        wasm_bindgen_futures::spawn_local(async move {
            set_status.set(LoadingState::Loading);

            match client
                .get::<_, Response<HealthcheckRes>>("/__meta/healthcheck", None::<()>)
                .await
            {
                Ok(res) => {
                    match res.payload {
                        Some(payload) => set_status.set(LoadingState::Loaded(payload.into())),
                        None => {
                            set_status.set(LoadingState::Error("empty healthcheck".to_string()))
                        },
                    }
                },
                Err(err) => set_status.set(LoadingState::Error(err.to_string())),
            }
        });
    });

    let dot = move || {
        match status.get() {
            LoadingState::Loaded(res) if res.healthy => "healthcheck-dot healthcheck-ok",
            LoadingState::Loaded(_) => "healthcheck-dot healthcheck-error",
            LoadingState::Loading => "healthcheck-dot healthcheck-loading",
            LoadingState::Error(_) => "healthcheck-dot healthcheck-error",
            LoadingState::Pending => "healthcheck-dot",
        }
    };

    view! {
        <div
            class="sidebar-healthcheck"
            title=move || match status.get() {
                LoadingState::Error(err) => err,
                LoadingState::Loaded(res) => format!("healthy: {}", res.healthy),
                _ => "checking…".to_string(),
            }
        >
            <span class=dot></span>

            <span class="healthcheck-version">
                {move || match status.get() {
                    LoadingState::Loaded(res) => res.version.clone(),
                    _ => String::new(),
                }}
            </span>
        </div>
    }
}
