use chrono::{DateTime, Utc};
use http::data::LoadingState;
use leptos::prelude::*;
use oxidauth_kernel::jwt::Jwt;

use crate::{
    features::dashboard::signals::{
        DashboardCounts,
        HandleDashboardResponse,
        handle_dashboard_signals,
    },
    json::json_text,
    state::AppState,
    time::format_local,
};

/// Stands in for a claim the token omits, so every row of the session card
/// keeps its label and the grid keeps its shape.
const MISSING: &str = "—";

/// An epoch-seconds claim (`iat`, `nbf`, `exp`) in the browser's own zone —
/// the rendering every other timestamp in the console gets (`crate::time`).
fn format_epoch(secs: usize) -> Option<String> {
    DateTime::from_timestamp(secs as i64, 0).map(|instant| format_local(&instant))
}

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

            <SessionCard />
        </div>
    }
}

/// The live token in full: every claim the console can see — whose session
/// this is, who signed it and for whom, the window it is valid for, its
/// context document, and what it grants. The one place an operator reads
/// their own token without opening devtools.
#[component]
fn SessionCard() -> impl IntoView {
    let state = expect_context::<AppState>();
    let session = state.session;

    // A claim row of the grid: `label` beside the text `select` digs out of
    // the token, or `MISSING` while the session is unknown or the claim is
    // absent.
    let claim = move |label: &'static str, select: fn(&Jwt) -> Option<String>| -> AnyView {
        view! {
            <>
                <dt>{label}</dt>
                <dd>
                    {move || {
                        session
                            .get()
                            .and_then(|jwt| select(&jwt))
                            .unwrap_or_else(|| MISSING.to_string())
                    }}
                </dd>
            </>
        }
        .into_any()
    };

    let user_id = move || {
        session
            .get()
            .and_then(|jwt| jwt.sub)
            .map(|sub| sub.to_string())
            .unwrap_or_else(|| MISSING.to_string())
    };

    // `exp` is the one claim never absent, and the one the SDK's refresh
    // loop turns over — so a stale token says so, in the forms' danger red,
    // instead of leaving the date to be read against the clock.
    let expires = move || {
        match session.get() {
            None => MISSING.to_string().into_any(),
            Some(jwt) if (jwt.exp as i64) <= Utc::now().timestamp() => {
                view! {
                    {format_epoch(jwt.exp).unwrap_or_else(|| MISSING.to_string())}
                    <span class="form-error">" (expired)"</span>
                }
                .into_any()
            },
            Some(jwt) => {
                format_epoch(jwt.exp)
                    .unwrap_or_else(|| MISSING.to_string())
                    .into_any()
            },
        }
    };

    // The context document as the show pages render every JSON block.
    let context = move || {
        match session
            .get()
            .and_then(|jwt| jwt.ctx)
        {
            Some(ctx) => view! { <pre>{json_text(&ctx)}</pre> }.into_any(),
            None => MISSING.to_string().into_any(),
        }
    };

    // The grants wear chips: a full permission tree wraps inside the card
    // instead of trailing off down a one-line-per-grant list.
    let entitlements = move || {
        let grants = state.entitlements();
        if grants.is_empty() {
            return MISSING.to_string().into_any();
        }

        view! {
            <ul class="chip-list">
                {grants
                    .into_iter()
                    .map(|permission| view! { <li>{permission}</li> })
                    .collect_view()}
            </ul>
        }
        .into_any()
    };

    view! {
        <div class="card session-card">
            <dl class="detail">
                <dt>"User"</dt>
                <dd class="record-id">{user_id}</dd>

                <div class="detail-rule"></div>

                {claim("Issuer", |jwt| jwt.iss.clone())}
                {claim("Audience", |jwt| jwt.aud.clone())}
                {claim("Issued", |jwt| jwt.iat.and_then(format_epoch))}
                {claim("Not before", |jwt| jwt.nbf.and_then(format_epoch))}

                <dt>"Expires"</dt>
                <dd>{expires}</dd>

                <div class="detail-rule"></div>

                <dt>"Context"</dt>
                <dd>{context}</dd>

                <div class="detail-rule"></div>

                <dt>"Entitlements"</dt>
                <dd>{entitlements}</dd>
            </dl>
        </div>
    }
}
