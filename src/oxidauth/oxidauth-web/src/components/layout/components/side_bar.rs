use leptos::prelude::*;
use leptos_router::components::A;

use crate::{
    components::logo::Logo,
    features::{auth::components::LogoutButton, healthcheck::components::Healthcheck},
    state::AppState,
};

/// Nav rows: (label, href, permission challenge). `""` always shows; the
/// rest appear only when the session grants them. The challenges are the
/// exact constants the api handlers enforce.
const NAV_ITEMS: &[(&str, &str, &str)] = &[
    ("Dashboard", "/dashboard", ""),
    ("Authorities", "/authorities", "oxidauth:authorities:manage"),
    ("Roles", "/roles", "oxidauth:roles:manage"),
    ("Permissions", "/permissions", "oxidauth:permissions:manage"),
    ("Users", "/users", "oxidauth:users:manage"),
    ("Invitations", "/invitations", "oxidauth:invitations:read"),
];

/// Sign out closes the nav (a `.nav-link` row pinned to the foot by the
/// footer's `margin-top: auto`); the divider and the api healthcheck sit
/// under it as status, not actions.
#[component]
pub fn SideBar() -> impl IntoView {
    let state = expect_context::<AppState>();

    view! {
        <aside class="sidebar" id="sidebar">
            <div class="sidebar-header">
                <A href="/dashboard" attr:class="logo">
                    <Logo class="logo-icon" />
                </A>
            </div>

            <nav class="sidebar-nav">
                <div class="nav-section">
                    <ul class="nav-items">
                        {move || {
                            // Tracked read: the row set rebuilds when the
                            // session lands (page-load bootstrap, login,
                            // logout). `can` itself reads untracked — it is
                            // built for click handlers, and without this
                            // read the rows render once against the not-yet-
                            // bootstrapped session and never update.
                            let _session = state.session.get();

                            NAV_ITEMS
                                .iter()
                                .filter(|(_, _, challenge)| {
                                    challenge.is_empty() || state.can(challenge)
                                })
                                .map(|(text, href, _)| {
                                    view! {
                                        <li class="nav-item">
                                            <A href=*href attr:class="nav-link">
                                                <span>{*text}</span>
                                            </A>
                                        </li>
                                    }
                                })
                                .collect_view()
                        }}
                    </ul>
                </div>
            </nav>

            <div class="sidebar-footer">
                <LogoutButton />
                <div class="sidebar-divider"></div>
                <Healthcheck />
            </div>
        </aside>
    }
}
