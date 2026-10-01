use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    components::A,
    hooks::{use_navigate, use_params},
    params::Params,
};
use oxidauth_kernel::roles::Role;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    components::{
        error_banner::ErrorBanner,
        pick_combo::{PickChoice, PickChoices, PickCombo},
    },
    features::roles::signals::{
        ChildRoleRow,
        HandleRoleFormResponse,
        HandleRolePermissionGrantsResponse,
        HandleRoleRoleGrantsResponse,
        HandleRolesListResponse,
        LoadingState,
        PermissionGrantRow,
        ROLES_MANAGE,
        SaveRoleForm,
        handle_role_detail_signals,
        handle_role_form_signals,
        handle_role_permission_grants_signals,
        handle_role_role_grants_signals,
        handle_roles_list_signals,
    },
    state::AppState,
    time::format_local,
};

/// `/roles/:id` and `/roles/:id/edit` path parameter. Empty on `/roles/new`,
/// which is exactly how the form page tells create from edit.
#[derive(Clone, Debug, PartialEq, Deserialize, Params)]
pub struct RoleIdParams {
    pub id: String,
}

fn use_id_param() -> Signal<String> {
    let params = use_params::<RoleIdParams>();

    Signal::derive(move || {
        params
            .get()
            .map(|params| params.id)
            .unwrap_or_default()
    })
}

/// Reactive form of the manage gate, for `Show`s and row buttons. The server
/// stays the enforcement point; this only hides what it would refuse.
fn can_manage_roles() -> Signal<bool> {
    expect_context::<AppState>().can_gate(ROLES_MANAGE)
}

#[component]
pub fn RolesPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let handlers = handle_roles_list_signals(state);
    let can_manage = can_manage_roles();

    view! {
        <div class="page-header">
            <h1 class="page-title">"Roles"</h1>

            <div class="page-actions">
                <Show when=move || can_manage.get() fallback=move || {}>
                    <A
                        href="/roles/new"
                        attr:class="button new-link"
                        attr:aria-label="New role"
                        attr:title="New role"
                    >
                        "+"
                    </A>
                </Show>
            </div>
        </div>

        <div class="list-filter">
            <label>
                "Search"
                <input
                    type="search"
                    placeholder="name"
                    prop:value=move || handlers.search.get()
                    on:input=move |ev| handlers.apply_search.run(event_target_value(&ev))
                />
            </label>
        </div>

        <ErrorBanner errors=handlers.errors />

        {move || -> AnyView {
            match handlers.roles.get() {
                LoadingState::Pending | LoadingState::Loading => {
                    view! { <p>"Loading..."</p> }.into_any()
                },
                LoadingState::Error(err) => view! { <p class="error">{err}</p> }.into_any(),
                LoadingState::Loaded(roles) => {
                    // The list is unpaginated server-side: the narrowing is
                    // a filter over the loaded rows, never a second query.
                    let needle = handlers.search.get().to_lowercase();

                    view! {
                        <table class="data-table">
                            <thead>
                                <tr>
                                    <th>"Name"</th>
                                    <th>"Created"</th>
                                    <th class="row-actions"></th>
                                </tr>
                            </thead>
                            <tbody>
                                {roles
                                    .into_iter()
                                    .filter(|role| {
                                        needle.is_empty()
                                            || role.name.to_lowercase().contains(&needle)
                                    })
                                    .map(|role| view! { <RoleRow role=role handlers=handlers /> })
                                    .collect_view()}
                            </tbody>
                        </table>
                    }
                    .into_any()
                },
            }
        }}

        {move || handlers.pending_delete.get().map(|pending| {
            view! {
                <div class="modal-overlay">
                    <div class="modal">
                        <h2 class="modal-title">"Delete role"</h2>
                        <p class="modal-message">
                            {format!(
                                "Delete \"{}\"? Its permission and child-role grants go with it, \
                                 and it cannot be edited again.",
                                pending.name,
                            )}
                        </p>
                        <div class="modal-actions">
                            <button
                                class="btn"
                                type="button"
                                on:click=move |_| handlers.cancel_delete.run(())
                            >
                                "Cancel"
                            </button>
                            <button
                                class="btn btn-danger"
                                type="button"
                                on:click=move |_| {
                                    handlers.confirm_delete.dispatch(pending.id.clone());
                                }
                            >
                                "Delete"
                            </button>
                        </div>
                    </div>
                </div>
            }
        })}
    }
}

#[component]
fn RoleRow(role: Role, handlers: HandleRolesListResponse) -> impl IntoView {
    let id = role.id.to_string();
    let href = format!("/roles/{id}");
    let name = role.name.clone();
    let created = format_local(&role.created_at);
    let can_manage = can_manage_roles();

    let navigate = use_navigate();
    let href_on_click = href.clone();
    let edit_href = format!("{href}/edit");

    // The delete pair rides a Copy callback: a gated button lives inside a
    // Show children closure that re-runs on every show/hide, and only Copy
    // handles — never Strings — can be captured across that boundary.
    let request_delete = Callback::new({
        let id = id.clone();
        let name = name.clone();

        move |_: ()| {
            handlers
                .request_delete
                .run((id.clone(), name.clone()))
        }
    });

    view! {
        <tr
            class="row-link"
            on:click=move |_| navigate(&href_on_click, NavigateOptions::default())
        >
            <td>{name}</td>
            <td class="cell-muted">{created}</td>
            <td class="row-actions">
                <Show when=move || can_manage.get() fallback=move || {}>
                    <A href=edit_href.clone() attr:class="btn">"Edit"</A>
                    <button
                        class="btn btn-danger"
                        type="button"
                        on:click=move |ev| {
                            ev.stop_propagation();
                            request_delete.run(());
                        }
                    >
                        "Delete"
                    </button>
                </Show>
            </td>
        </tr>
    }
}

/// The role card plus the two grant sections. The sections key off the route
/// id themselves, so they fetch and refetch without this page coordinating.
#[component]
pub fn RoleDetailPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let role_id = use_id_param();
    let handlers = handle_role_detail_signals(state, role_id);
    let can_manage = can_manage_roles();

    view! {
        <p class="back-link"><A href="/roles">"Back to roles"</A></p>

        <ErrorBanner errors=handlers.errors />

        {move || -> AnyView {
            match handlers.role.get() {
                LoadingState::Pending | LoadingState::Loading => {
                    view! { <p>"Loading..."</p> }.into_any()
                },
                LoadingState::Error(err) => view! { <p class="error">{err}</p> }.into_any(),
                LoadingState::Loaded(role) => {
                    let id = role.id.to_string();
                    let edit_href = format!("/roles/{id}/edit");
                    let created = format_local(&role.created_at);
                    let updated = format_local(&role.updated_at);

                    view! {
                        <div class="page-header">
                            <h1 class="page-title">{role.name}</h1>

                            <div class="page-actions">
                                <Show when=move || can_manage.get() fallback=move || {}>
                                    <A href=edit_href.clone() attr:class="button">"Edit"</A>
                                </Show>
                            </div>
                        </div>

                        <section class="card">
                            <dl class="detail">
                                <dt>"Id"</dt>
                                <dd class="record-id">{id}</dd>

                                <dt>"Created"</dt>
                                <dd>{created}</dd>

                                <dt>"Updated"</dt>
                                <dd>{updated}</dd>
                            </dl>
                        </section>
                    }
                    .into_any()
                },
            }
        }}

        <RolePermissionGrantsSection role_id />
        <RoleRoleGrantsSection role_id />
    }
}

/// New and edit share one page, as the routing table demands: no `:id`
/// means create, an `:id` preloads the role and the save updates it.
#[component]
pub fn RoleFormPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let handlers: HandleRoleFormResponse = handle_role_form_signals(state, use_id_param());

    let (name, set_name) = signal(String::new());
    let save = handlers.save;
    let saving = save.pending();
    let navigate = use_navigate();
    let can_manage = can_manage_roles();

    // The edit preload lands in the field once per load; create never
    // triggers this arm.
    let role_state = handlers.role;
    Effect::new(move |_| {
        if let LoadingState::Loaded(Some(role)) = role_state.get() {
            set_name.set(role.name);
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(saved_id)) = save.value().get() {
            navigate(&format!("/roles/{saved_id}"), NavigateOptions::default());
        }
    });

    view! {
        <p class="back-link"><A href="/roles">"Back to roles"</A></p>

        <ErrorBanner errors=handlers.errors />

        {move || -> AnyView {
            match handlers.role.get() {
                LoadingState::Pending | LoadingState::Loading => {
                    view! { <p>"Loading..."</p> }.into_any()
                },
                LoadingState::Error(err) => view! { <p class="error">{err}</p> }.into_any(),
                LoadingState::Loaded(existing) => {
                    let editing = existing.is_some();

                    view! {
                        <h1 class="page-title">{if editing { "Edit role" } else { "New role" }}</h1>

                        <div class="form">
                            <label class="form-label">
                                <span>"Name"</span>
                                <input
                                    class="form-input"
                                    type="text"
                                    prop:value=move || name.get()
                                    on:input=move |ev| set_name.set(event_target_value(&ev))
                                />
                            </label>

                            <Show when=move || can_manage.get() fallback=move || {}>
                                <button
                                    class="btn btn-primary"
                                    type="button"
                                    disabled=move || saving.get()
                                    on:click=move |_| {
                                        // The id comes from the preloaded role, not a
                                        // captured String: a `move |_|` handler that took
                                        // `id` by value would make the enclosing Show
                                        // children closure FnOnce.
                                        let id = match role_state.get_untracked() {
                                            LoadingState::Loaded(Some(role)) => role.id.to_string(),
                                            _ => String::new(),
                                        };

                                        save.dispatch(SaveRoleForm {
                                            id,
                                            name: name.get(),
                                        });
                                    }
                                >
                                    {move || {
                                        if saving.get() {
                                            "Saving..."
                                        } else if editing {
                                            "Save"
                                        } else {
                                            "Create"
                                        }
                                    }}
                                </button>
                            </Show>
                        </div>
                    }
                    .into_any()
                },
            }
        }}
    }
}

/// Granted permissions with a pick-and-grant row over the full catalog.
#[component]
fn RolePermissionGrantsSection(role_id: Signal<String>) -> impl IntoView {
    let state = expect_context::<AppState>();
    let handlers: HandleRolePermissionGrantsResponse =
        handle_role_permission_grants_signals(state, role_id);
    let can_manage = can_manage_roles();

    let (selected, set_selected) = signal(String::new());
    let grant = handlers.grant;
    let granting = grant.pending();

    // The pick is spent once the grant lands (or is refused), so the box
    // returns to its placeholder each time.
    Effect::new(move |_| {
        if grant.value().get().is_some() {
            set_selected.set(String::new());
        }
    });

    let choices = Signal::derive(move || {
        handlers
            .catalog
            .with(|catalog| {
                match catalog {
                    LoadingState::Loaded(catalog) => {
                        handlers
                            .grants
                            .with(|grants| {
                                match grants {
                                    LoadingState::Loaded(grants) => {
                                        let granted: Vec<String> = grants
                                            .iter()
                                            .map(|row| row.permission.clone())
                                            .collect();

                                        PickChoices::Ready(
                                            catalog
                                                .iter()
                                                .map(|permission| {
                                                    PickChoice::same(permission.to_string())
                                                })
                                                .filter(|choice| !granted.contains(&choice.value))
                                                .collect(),
                                        )
                                    },
                                    LoadingState::Error(err) => {
                                        PickChoices::Error(format!("options unavailable: {err}"))
                                    },
                                    _ => PickChoices::Loading,
                                }
                            })
                    },
                    LoadingState::Error(err) => {
                        PickChoices::Error(format!("options unavailable: {err}"))
                    },
                    _ => PickChoices::Loading,
                }
            })
    });

    view! {
        <div class="section card">
            <h2 class="section-title">"Permissions"</h2>

            <ErrorBanner errors=handlers.errors />

            {move || -> AnyView {
                match handlers.grants.get() {
                    LoadingState::Pending | LoadingState::Loading => {
                        view! { <p>"Loading..."</p> }.into_any()
                    },
                    LoadingState::Error(err) => {
                        view! { <p class="error">{err}</p> }.into_any()
                    },
                    LoadingState::Loaded(rows) => view! {
                        <table class="data-table">
                            <thead>
                                <tr>
                                    <th>"Permission"</th>
                                    <th>"Granted"</th>
                                    <th class="row-actions"></th>
                                </tr>
                            </thead>
                            <tbody>
                                {rows
                                    .into_iter()
                                    .map(|row| {
                                        view! {
                                            <PermissionGrantRowView
                                                row=row
                                                revoke=handlers.revoke
                                                can_manage=can_manage
                                            />
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    }
                    .into_any(),
                }
            }}

            <Show when=move || can_manage.get() fallback=move || {}>
                <div class="create-row">
                    <label>
                        "Permission"
                        <PickCombo
                            choices=choices
                            placeholder="-- choose a permission --"
                            loading_text="loading permissions..."
                            empty_text="every permission is granted"
                            selected=selected.into()
                            on_pick=Callback::new(move |value: String| set_selected.set(value))
                        />
                    </label>

                    <button
                        type="button"
                        disabled=move || selected.get().is_empty() || granting.get()
                        on:click=move |_| {
                            let permission = selected.get_untracked();
                            if !permission.is_empty() {
                                grant.dispatch(permission);
                            }
                        }
                    >
                        "Grant"
                    </button>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn PermissionGrantRowView(
    row: PermissionGrantRow,
    revoke: Action<String, ()>,
    can_manage: Signal<bool>,
) -> impl IntoView {
    let permission_text = row.permission.clone();
    let granted = format_local(&row.granted_at);

    // The revoke text rides a Copy callback — see the note in RoleRow.
    let revoke_grant = Callback::new(move |_: ()| revoke.dispatch(permission_text.clone()));

    view! {
        <tr>
            <td class="record-id">{row.permission}</td>
            <td class="cell-muted">{granted}</td>
            <td class="row-actions">
                <Show when=move || can_manage.get() fallback=move || {}>
                    <button
                        class="btn btn-danger"
                        type="button"
                        on:click=move |_| { revoke_grant.run(()); }
                    >
                        "Revoke"
                    </button>
                </Show>
            </td>
        </tr>
    }
}

/// Child roles with a pick-and-grant row over the full role list, minus
/// this role itself and the already-granted children. A pick that would
/// close a cycle is refused by the server and shown here verbatim.
#[component]
fn RoleRoleGrantsSection(role_id: Signal<String>) -> impl IntoView {
    let state = expect_context::<AppState>();
    let handlers: HandleRoleRoleGrantsResponse = handle_role_role_grants_signals(state, role_id);
    let can_manage = can_manage_roles();

    let (selected, set_selected) = signal(String::new());
    let grant = handlers.grant;
    let granting = grant.pending();

    Effect::new(move |_| {
        if grant.value().get().is_some() {
            set_selected.set(String::new());
        }
    });

    let choices = Signal::derive(move || {
        let parent = role_id.get();

        handlers
            .catalog
            .with(|catalog| {
                match catalog {
                    LoadingState::Loaded(catalog) => {
                        handlers
                            .grants
                            .with(|grants| {
                                match grants {
                                    LoadingState::Loaded(grants) => {
                                        let child_ids: Vec<Uuid> = grants
                                            .iter()
                                            .map(|row| row.role.id)
                                            .collect();

                                        PickChoices::Ready(
                                            catalog
                                                .iter()
                                                .filter(|role| {
                                                    role.id.to_string() != parent
                                                        && !child_ids.contains(&role.id)
                                                })
                                                .map(|role| {
                                                    PickChoice::new(
                                                        role.id.to_string(),
                                                        role.name.clone(),
                                                    )
                                                })
                                                .collect(),
                                        )
                                    },
                                    LoadingState::Error(err) => {
                                        PickChoices::Error(format!("options unavailable: {err}"))
                                    },
                                    _ => PickChoices::Loading,
                                }
                            })
                    },
                    LoadingState::Error(err) => {
                        PickChoices::Error(format!("options unavailable: {err}"))
                    },
                    _ => PickChoices::Loading,
                }
            })
    });

    view! {
        <div class="section card">
            <h2 class="section-title">"Child roles"</h2>

            <ErrorBanner errors=handlers.errors />

            {move || -> AnyView {
                match handlers.grants.get() {
                    LoadingState::Pending | LoadingState::Loading => {
                        view! { <p>"Loading..."</p> }.into_any()
                    },
                    LoadingState::Error(err) => {
                        view! { <p class="error">{err}</p> }.into_any()
                    },
                    LoadingState::Loaded(rows) => view! {
                        <table class="data-table">
                            <thead>
                                <tr>
                                    <th>"Role"</th>
                                    <th>"Granted"</th>
                                    <th class="row-actions"></th>
                                </tr>
                            </thead>
                            <tbody>
                                {rows
                                    .into_iter()
                                    .map(|row| {
                                        view! {
                                            <ChildRoleGrantRowView
                                                row=row
                                                revoke=handlers.revoke
                                                can_manage=can_manage
                                            />
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    }
                    .into_any(),
                }
            }}

            <Show when=move || can_manage.get() fallback=move || {}>
                <div class="create-row">
                    <label>
                        "Child role"
                        <PickCombo
                            choices=choices
                            placeholder="-- choose a role --"
                            loading_text="loading roles..."
                            empty_text="no role left to grant"
                            selected=selected.into()
                            on_pick=Callback::new(move |value: String| set_selected.set(value))
                        />
                    </label>

                    <button
                        type="button"
                        disabled=move || selected.get().is_empty() || granting.get()
                        on:click=move |_| {
                            let child_id = selected.get_untracked();
                            if !child_id.is_empty() {
                                grant.dispatch(child_id);
                            }
                        }
                    >
                        "Add child"
                    </button>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn ChildRoleGrantRowView(
    row: ChildRoleRow,
    revoke: Action<String, ()>,
    can_manage: Signal<bool>,
) -> impl IntoView {
    let id = row.role.id.to_string();
    let href = format!("/roles/{id}");
    let name = row.role.name.clone();
    let granted = format_local(&row.granted_at);

    let navigate = use_navigate();
    let href_on_click = href.clone();

    // The child id rides a Copy callback — see the note in RoleRow.
    let revoke_child = Callback::new(move |_: ()| revoke.dispatch(id.clone()));

    view! {
        <tr
            class="row-link"
            on:click=move |_| navigate(&href_on_click, NavigateOptions::default())
        >
            <td>{name}</td>
            <td class="cell-muted">{granted}</td>
            <td class="row-actions">
                <Show when=move || can_manage.get() fallback=move || {}>
                    <button
                        class="btn btn-danger"
                        type="button"
                        on:click=move |_| { revoke_child.run(()); }
                    >
                        "Revoke"
                    </button>
                </Show>
            </td>
        </tr>
    }
}
