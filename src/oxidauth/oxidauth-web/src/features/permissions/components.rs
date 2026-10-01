use http::data::LoadingState;
use leptos::prelude::*;
use oxidauth_kernel::permissions::Permission;

use crate::{
    components::error_banner::ErrorBanner,
    features::permissions::signals::{HandlePermissionsResponse, handle_permissions_signals},
    state::AppState,
    time::format_local,
};

/// The challenge the api demands of every permission write; the buttons here
/// only hide what the server would refuse anyway.
const MANAGE: &str = "oxidauth:permissions:manage";

/// `/permissions` — every permission in existence, named by its full
/// `realm:resource:action` key. Rows are added by typing that key and removed
/// through a confirmation; the api has no permission update, so a key that
/// needs changing is deleted and added again.
#[component]
pub fn PermissionsPage() -> impl IntoView {
    let state = expect_context::<AppState>();

    let handlers = handle_permissions_signals(state);

    let can_manage = state.can_gate(MANAGE);
    let creating = handlers.create.pending();

    // An empty list and an emptied filter are different dead ends, and the
    // page knows which one it is showing.
    let empty_message = move || {
        if handlers
            .search
            .get()
            .is_empty()
        {
            "No permissions yet."
        } else {
            "No permission matches that filter."
        }
    };

    view! {
        <div class="page-header">
            <h1 class="page-title">"Permissions"</h1>
        </div>

        <ErrorBanner errors=handlers.errors />

        {move || can_manage.get().then(|| view! {
            <div>
                <form
                    class="create-row"
                    on:submit=move |ev| {
                        ev.prevent_default();
                        handlers.create.dispatch(handlers.key.get());
                    }
                >
                    <label>
                        "Permission"
                        <input
                            type="text"
                            prop:value=move || handlers.key.get()
                            on:input=move |ev| handlers.set_key.set(event_target_value(&ev))
                            placeholder="oxidauth:users:read"
                        />
                    </label>

                    <button
                        type="submit"
                        class="plus-action"
                        aria-label="Add permission"
                        title="Add permission"
                        disabled=move || creating.get()
                    >
                        "+"
                    </button>
                </form>

                <ErrorBanner errors=handlers.form_errors />
            </div>
        })}

        <div class="list-filter">
            <label>
                "Filter"
                <input
                    type="search"
                    prop:value=move || handlers.search.get()
                    on:input=move |ev| handlers.apply_search.run(event_target_value(&ev))
                    placeholder="realm, resource or action"
                />
            </label>
        </div>

        {move || -> AnyView {
            match handlers.permissions.get() {
                LoadingState::Pending | LoadingState::Loading => {
                    view! { <p>"Loading..."</p> }.into_any()
                },
                LoadingState::Error(err) => view! { <p class="error">{err}</p> }.into_any(),
                LoadingState::Loaded(_) => view! {
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>"Permission"</th>
                                <th>"Created"</th>
                                <th class="row-actions"></th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let rows = handlers.visible.get();

                                if rows.is_empty() {
                                    return view! {
                                        <tr>
                                            <td colspan="3" class="cell-muted">
                                                {empty_message()}
                                            </td>
                                        </tr>
                                    }
                                    .into_any();
                                }

                                rows.into_iter()
                                    .map(|permission| {
                                        view! {
                                            <PermissionRow
                                                permission=permission
                                                can_manage=can_manage
                                                handlers=handlers
                                            />
                                        }
                                    })
                                    .collect_view()
                                    .into_any()
                            }}
                        </tbody>
                    </table>
                }
                .into_any(),
            }
        }}

        {move || {
            handlers
                .pending_delete
                .get()
                .map(|pending| {
                    let key = pending.key;
                    let shown = key.clone();

                    view! {
                        <div
                            class="modal-overlay"
                            on:click=move |_| handlers.cancel_delete.run(())
                        >
                            <div class="modal" on:click=move |ev| ev.stop_propagation()>
                                <h2 class="modal-title">"Delete permission"</h2>

                                <p class="modal-message">
                                    {format!(
                                        "Delete \"{shown}\"? Users and roles that hold it lose that action.",
                                    )}
                                </p>

                                <div class="modal-actions">
                                    <button
                                        type="button"
                                        on:click=move |_| handlers.cancel_delete.run(())
                                    >
                                        "Cancel"
                                    </button>

                                    <button
                                        type="button"
                                        class="btn btn-danger"
                                        on:click=move |_| {
                                            handlers.confirm_delete.dispatch(key.clone());
                                        }
                                    >
                                        "Delete"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                })
        }}
    }
}

/// One permission row: the key as an entitlement line spells it, its age, and
/// the delete that takes that same key as its path parameter.
#[component]
fn PermissionRow(
    permission: Permission,
    can_manage: Signal<bool>,
    handlers: HandlePermissionsResponse,
) -> impl IntoView {
    let key = permission.to_string();
    let created = format_local(&permission.created_at);
    let for_delete = key.clone();

    view! {
        <tr>
            <td>
                <span class="record-id">{key}</span>
            </td>

            <td class="cell-muted">{created}</td>

            <td class="row-actions">
                {move || {
                    let for_delete = for_delete.clone();

                    can_manage.get().then(move || view! {
                        <button
                            type="button"
                            class="btn btn-danger"
                            on:click=move |_| handlers.request_delete.run(for_delete.clone())
                        >
                            "Delete"
                        </button>
                    })
                }}
            </td>
        </tr>
    }
}
