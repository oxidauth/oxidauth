use http::data::LoadingState;
use leptos::prelude::*;

use crate::{
    features::dashboard::signals::{
        DashboardCounts,
        HandleDashboardResponse,
        handle_dashboard_signals,
    },
    state::AppState,
};

/// Overview: record counts per managed domain plus the current session
/// (who am I, what may I do) — the console's landing state at a glance.
#[component]
pub fn DashboardPage() -> impl IntoView {
    let state = expect_context::<AppState>();

    let HandleDashboardResponse { counts, refresh } = handle_dashboard_signals(state);

    let card = move |label: &'static str, total: fn(DashboardCounts) -> usize| -> AnyView {
        view! {
            <div class="metric-card">
                <span class="metric-value">
                    {move || match counts.get() {
                        LoadingState::Loaded(counts) => total(counts).to_string(),
                        LoadingState::Loading | LoadingState::Pending => "…".to_string(),
                        LoadingState::Error(_) => "!".to_string(),
                    }}
                </span>
                <span class="metric-label">{label}</span>
            </div>
        }
        .into_any()
    };

    let entitlements = move || {
        state
            .entitlements()
            .into_iter()
            .map(|permission| view! { <li>{permission}</li> })
            .collect_view()
    };

    let user_id = move || {
        state
            .session
            .get()
            .and_then(|jwt| jwt.sub)
            .map(|sub| sub.to_string())
            .unwrap_or_default()
    };

    view! {
        <div class="page-header">
            <h1 class="page-title">"Dashboard"</h1>

            <div class="page-actions">
                <button
                    class="btn"
                    type="button"
                    on:click=move |_| {
                        refresh.dispatch(());
                    }
                >
                    "Refresh"
                </button>
            </div>
        </div>

        <div class="metric-cards">
            {card("Users", |counts| counts.users)}
            {card("Authorities", |counts| counts.authorities)}
            {card("Roles", |counts| counts.roles)}
            {card("Permissions", |counts| counts.permissions)}
        </div>

        <div class="section">
            <h2 class="section-title">"This session"</h2>

            <div class="card detail">
                <div class="detail-list">
                    <div class="detail-row">
                        <span class="detail-label">"User"</span>
                        <span class="detail-value record-id">{user_id}</span>
                    </div>
                </div>

                <div class="detail-rule"></div>

                <h3 class="section-title">"Entitlements"</h3>

                <ul>{entitlements}</ul>
            </div>
        </div>
    }
}
