use http::data::LoadingState;
use leptos::{logging::log, prelude::*};
use oxidauth::prelude::*;
use oxidauth_kernel::permissions::{
    Permission,
    RawPermission,
    create_permission::CreatePermission,
    delete_permission::DeletePermission,
    list_all_permissions::ListAllPermissions,
};
// `oxidauth_permission::parse` is the module; the grammar lives in it.
use oxidauth_permission::parse::parse;

use crate::{
    features::auth::signals::{handle_feature_error, logout_callback},
    state::AppState,
};

/// The row a pending delete confirmation refers to. A permission's key is both
/// its label and — the api has no id-shaped delete — its path parameter, so
/// the modal shows and sends the same text.
#[derive(Clone, Debug, PartialEq)]
pub struct PendingDelete {
    pub key: String,
}

/// Signals for the one permissions screen: the whole list, plus the two
/// mutations the api offers. There is no update endpoint, so nothing here
/// edits a row in place.
#[derive(Clone, Copy)]
pub struct HandlePermissionsResponse {
    /// Every permission the api holds. The endpoint is unpaginated and takes
    /// no filter, so the set is fetched once and narrowed in the page.
    pub permissions: ReadSignal<LoadingState<Vec<Permission>>>,

    /// The rows the filter leaves standing, in the order the api sent them.
    pub visible: Signal<Vec<Permission>>,
    pub search: ReadSignal<String>,
    pub apply_search: Callback<String>,

    /// The create field. It lives here so a successful add can clear it.
    pub key: ReadSignal<String>,
    pub set_key: WriteSignal<String>,
    pub create: Action<String, ()>,
    /// Why the last add was refused: a malformed key, or the api's answer.
    pub form_errors: ReadSignal<Option<Vec<String>>>,

    /// Page-level failures (a refused delete). Fetch failures live in
    /// [`LoadingState::Error`] on the list itself.
    pub errors: ReadSignal<Option<Vec<String>>>,

    pub pending_delete: ReadSignal<Option<PendingDelete>>,
    pub request_delete: Callback<String>,
    pub cancel_delete: Callback<()>,
    pub confirm_delete: Action<String, ()>,
}

pub fn handle_permissions_signals(state: AppState) -> HandlePermissionsResponse {
    let (permissions, set_permissions) = signal(LoadingState::<Vec<Permission>>::Pending);
    let (search, set_search) = signal(String::new());
    let (key, set_key) = signal(String::new());
    let (form_errors, set_form_errors) = signal(None::<Vec<String>>);
    let (errors, set_errors) = signal(None::<Vec<String>>);
    let (pending_delete, set_pending_delete) = signal(None::<PendingDelete>);

    let fetch = Action::new_unsync(move |_: &()| {
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_permissions.set(LoadingState::Loading);

        async move {
            match client
                .list_all_permissions(ListAllPermissions)
                .await
            {
                Ok(res) => set_permissions.set(LoadingState::Loaded(res.permissions)),
                Err(err) => {
                    log!("error fetching permissions: {err}");

                    // A dead session is already on its way to the login page —
                    // the redirect is the message, so the list simply stops
                    // loading rather than showing text nobody will read.
                    if let Some(message) = handle_feature_error(err.as_ref(), &logout) {
                        set_permissions.set(LoadingState::Error(message));
                    }
                },
            };
        }
    });

    Effect::new(move |_| {
        fetch.dispatch(());
    });

    // The narrowing is local, so nothing re-fetches: it only rewrites the
    // derived rows.
    let apply_search = Callback::new(move |text: String| {
        set_search.set(text.trim().to_string());
    });

    let visible = Signal::derive(move || {
        let rows = match permissions.get() {
            LoadingState::Loaded(rows) => rows,
            _ => Vec::new(),
        };

        let needle = search.get().to_lowercase();

        if needle.is_empty() {
            return rows;
        }

        rows.into_iter()
            // One field is filtered, because one field is the row: the key is
            // realm, resource and action together.
            .filter(|permission| permission.to_string().to_lowercase().contains(&needle))
            .collect()
    });

    let create = Action::new_unsync(move |text: &String| {
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_form_errors.set(None);

        // A malformed key is refused here, so it never becomes a request.
        let submitted = match validate_key(text) {
            Ok(permission) => Some(permission),
            Err(message) => {
                set_form_errors.set(Some(vec![message]));
                None
            },
        };

        async move {
            let Some(permission) = submitted else {
                return;
            };

            match client
                .create_permission(CreatePermission { permission })
                .await
            {
                Ok(_) => {
                    set_key.set(String::new());
                    fetch.dispatch(());
                },
                Err(err) => {
                    if let Some(message) = handle_feature_error(err.as_ref(), &logout) {
                        set_form_errors.set(Some(vec![message]));
                    }
                },
            };
        }
    });

    let request_delete = Callback::new(move |key: String| {
        set_pending_delete.set(Some(PendingDelete { key }));
    });

    let cancel_delete = Callback::new(move |_: ()| {
        set_pending_delete.set(None);
    });

    let confirm_delete = Action::new_unsync(move |key: &String| {
        let key = key.clone();
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        // One failure's text never outlives the attempt that wrote it.
        set_errors.set(None);

        async move {
            match client
                .delete_permission(DeletePermission { permission: key })
                .await
            {
                Ok(_) => {
                    set_pending_delete.set(None);
                    fetch.dispatch(());
                },
                Err(err) => {
                    set_pending_delete.set(None);

                    if let Some(message) = handle_feature_error(err.as_ref(), &logout) {
                        set_errors.set(Some(vec![message]));
                    }
                },
            };
        }
    });

    HandlePermissionsResponse {
        permissions,
        visible,
        search,
        apply_search,
        key,
        set_key,
        create,
        form_errors,
        errors,
        pending_delete,
        request_delete,
        cancel_delete,
        confirm_delete,
    }
}

/// The whole key as the api should receive it, or the reason the form refuses
/// it. Two checks, both run server-side too: the `oxidauth-permission` grammar
/// (`parse`) says whether the text is a permission at all — its alphabet, at
/// most two separators — and the api's own [`RawPermission`] conversion then
/// needs three non-empty parts to store a row, which `parse` alone does not
/// (it accepts a bare `oxidauth`, and a trailing `oxidauth:users:`).
fn validate_key(text: &str) -> Result<String, String> {
    let key = text.trim();

    if key.is_empty() {
        return Err("enter a permission as realm:resource:action".to_string());
    }

    parse(key).map_err(|err| format!("\"{key}\" is not a valid permission: {err}"))?;

    RawPermission::try_from(key)?;

    Ok(key.to_string())
}
