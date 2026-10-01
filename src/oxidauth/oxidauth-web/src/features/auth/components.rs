use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::{
    components::logo::Logo,
    features::auth::signals::{HandleLoginResponse, handle_login_signals},
    state::AppState,
};

/// The api origin refused to build its client (missing/malformed
/// `OXIDAUTH_API_URL` / `OXIDAUTH_CLIENT_KEY` at build time). Shown instead
/// of a blank page so a mis-configured deployment says why.
#[component]
pub fn ConfigErrorPage(message: String) -> impl IntoView {
    view! {
        <div class="login-page">
            <div class="login-card">
                <div class="logo">
                    <Logo class="logo-icon" />
                </div>
                <h1 class="page-title">"oxidauth console"</h1>
                <p class="form-error">{message}</p>
            </div>
        </div>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let state = expect_context::<AppState>();

    let HandleLoginResponse { login, errors } = handle_login_signals(state);

    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());

    // An alive session has no business on the login form.
    Effect::new(move |_| {
        if state.session.get().is_some() {
            use_navigate()("/dashboard", Default::default());
        }
    });

    let submitting = login.pending();

    view! {
        <div class="login-page">
            <form
                class="login-card"
                on:submit=move |ev| {
                    ev.prevent_default();
                    login.dispatch((username.get(), password.get()));
                }
            >
                <div class="logo">
                    <Logo class="logo-icon" />
                </div>

                <h1 class="page-title">"Sign in"</h1>

                <Show
                    when=move || errors.get().is_some()
                    fallback=move || {}
                >
                    <div class="form-errors">
                        {move || {
                            errors
                                .get()
                                .unwrap_or_default()
                                .into_iter()
                                .map(|err| view! { <p class="form-error">{err}</p> })
                                .collect_view()
                        }}
                    </div>
                </Show>

                <label>
                    <span>"Username"</span>
                    <input
                        type="text"
                        autocomplete="username"
                        prop:value=move || username.get()
                        on:input=move |ev| set_username.set(event_target_value(&ev))
                    />
                </label>

                <label>
                    <span>"Password"</span>
                    <input
                        type="password"
                        autocomplete="current-password"
                        prop:value=move || password.get()
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                    />
                </label>

                <button class="button" type="submit" disabled=move || submitting.get()>
                    {move || if submitting.get() { "Signing in..." } else { "Sign in" }}
                </button>
            </form>
        </div>
    }
}

/// Renders children only while a session may be live. The redirect waits
/// for `booted` — the page-load revalidation — so reloading a protected
/// page as a signed-in user never flashes the login bounce while the SDK is
/// still verifying/refreshing the stored JWT (parkinglot's shell shape).
#[component]
pub fn Protected(children: Children) -> impl IntoView {
    let state = expect_context::<AppState>();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if state.booted.get() && state.session.get().is_none() {
            navigate("/login", Default::default());
        }
    });

    view! { {children()} }
}

/// A nav row closing the sidebar: styled as `.nav-link` so signing out
/// reads as part of the nav rather than a stranded form button.
#[component]
pub fn LogoutButton() -> impl IntoView {
    let state = expect_context::<AppState>();

    let logout = crate::features::auth::signals::logout_callback(state);

    view! {
        <button class="nav-link" type="button" on:click=move |_| logout.run(())>
            <span>"Sign out"</span>
        </button>
    }
}
