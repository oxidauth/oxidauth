use leptos::{logging::log, prelude::*};
use leptos_router::hooks::use_navigate;

use crate::state::{AppState, is_session_dead};

pub struct HandleLoginResponse {
    pub login: Action<(String, String), ()>,
    pub errors: ReadSignal<Option<Vec<String>>>,
}

pub fn handle_login_signals(state: AppState) -> HandleLoginResponse {
    let (errors, set_errors) = signal(None);
    let navigate = use_navigate();

    let login = Action::new_unsync(move |(username, password): &(String, String)| {
        let username = username.clone();
        let password = password.clone();
        let navigate = navigate.clone();
        let client = state.client.get_untracked();

        set_errors.set(None);

        async move {
            match client
                .authenticate(&username, &password)
                .await
            {
                Ok(true) => {
                    match client.get_jwt_decoded().await {
                        Ok(jwt) => {
                            state.session.set(Some(jwt));
                            state.booted.set(true);
                            navigate("/dashboard", Default::default());
                        },
                        Err(err) => {
                            log!("authenticated but the jwt failed validation: {err}");
                            set_errors.set(Some(vec![err.to_string()]));
                        },
                    }
                },
                Ok(false) => set_errors.set(Some(vec!["authentication failed".to_string()])),
                Err(err) => {
                    log!("login failed: {err}");
                    set_errors.set(Some(vec![err.to_string()]));
                },
            };
        }
    });

    HandleLoginResponse { login, errors }
}

/// Clears the session (SDK logout wipes memory + LocalStorage) and bounces to
/// the login page. Stored as a callback so feature actions — which see a dead
/// session mid-request — can end it without a router context of their own.
pub fn logout_callback(state: AppState) -> Callback<()> {
    let navigate = use_navigate();

    Callback::new(move |_: ()| {
        let client = state.client.get_untracked();

        wasm_bindgen_futures::spawn_local(async move {
            client.logout().await;
        });

        state.session.set(None);
        state.booted.set(true);
        navigate("/login", Default::default());
    })
}

/// The single error sink every feature action uses: a session-dead error
/// ends the session and is swallowed (the redirect is the message); anything
/// else comes back as displayable text.
pub fn handle_feature_error(
    err: &(dyn std::error::Error + 'static),
    logout: &Callback<()>,
) -> Option<String> {
    if is_session_dead(err) {
        logout.run(());
        None
    } else {
        Some(err.to_string())
    }
}
