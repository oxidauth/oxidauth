//! The users pages: the list with its browser-side filters, the create/edit
//! form, and the detail page carrying the three grant sections a user owns.
//!
//! Views only: every call, parse, and refetch lives in
//! [`crate::features::users::signals`], and mutating affordances appear only
//! for a session that holds `oxidauth:users:manage` — the server re-checks the
//! same challenge on every route.

use leptos::{html, prelude::*};
use leptos_router::{components::A, hooks::use_params, params::Params};
use serde::Deserialize;

use crate::{
    components::{
        error_banner::ErrorBanner,
        pick_combo::{PickChoice, PickChoices, PickCombo},
    },
    features::users::signals::{
        EditUserForm,
        LoadingState,
        NewUserForm,
        USER_KINDS,
        USER_STATUSES,
        USERS_MANAGE,
        full_name,
        handle_user_authorities_signals,
        handle_user_detail_signals,
        handle_user_edit_signals,
        handle_user_new_signals,
        handle_user_permissions_signals,
        handle_user_roles_signals,
        handle_users_list_signals,
        profile_text,
        user_matches,
    },
    state::AppState,
    time::format_local,
};

/// `/users/:id` and `/users/:id/edit` path parameter.
#[derive(Clone, Debug, PartialEq, Deserialize, Params)]
pub struct UserIdParams {
    pub id: String,
}

/// The id in the route, as one signal: the empty string is `/users/new`,
/// where there is no user to name.
fn use_id_param() -> Signal<String> {
    let params = use_params::<UserIdParams>();

    Signal::derive(move || {
        params
            .get()
            .map(|params| params.id)
            .unwrap_or_default()
    })
}

/// Whether this session may change users at all.
fn can_manage(state: AppState) -> Signal<bool> {
    state.can_gate(USERS_MANAGE)
}

/// The one message a session without the challenge sees in place of a button.
#[component]
fn NotPermitted() -> impl IntoView {
    view! { <p class="cell-muted">"This session cannot change users."</p> }
}

// -------------------------------------------------------------- the list

/// `/users`: every user, narrowed in the browser over username, email and the
/// two names plus one status. There is no pager because the api returns the
/// whole set.
#[component]
pub fn UsersPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let handlers = handle_users_list_signals(state);
    let can_manage = can_manage(state);

    view! {
        <div class="page-header">
            <h1 class="page-title">"Users"</h1>

            <div class="page-actions">
                <Show when=move || can_manage.get() fallback=|| {}>
                    <A
                        href="/users/new"
                        attr:class="button new-link"
                        attr:aria-label="New user"
                        attr:title="New user"
                    >
                        "+"
                    </A>
                </Show>
            </div>
        </div>

        <div class="list-filter">
            <label>
                <span>"Search"</span>
                <input
                    type="search"
                    placeholder="username, email, or name"
                    on:input=move |ev| {
                        handlers.apply_search.run(event_target_value(&ev));
                    }
                />
            </label>

            <label>
                <span>"Status"</span>
                <select on:change=move |ev| {
                    handlers.apply_status.run(event_target_value(&ev));
                }>
                    <option value="">"all"</option>
                    {USER_STATUSES
                        .into_iter()
                        .map(|status| view! { <option value=status>{status}</option> })
                        .collect_view()}
                </select>
            </label>
        </div>

        <ErrorBanner errors=handlers.errors />

        {move || -> AnyView {
            let search = handlers.search.get();
            let status = handlers.status.get();

            handlers.users.with(|loaded| match loaded {
                LoadingState::Pending | LoadingState::Loading => {
                    view! { <p class="cell-muted">"Loading..."</p> }.into_any()
                },
                LoadingState::Error(err) => view! { <p class="error">{err.clone()}</p> }.into_any(),
                LoadingState::Loaded(users) => {
                    let visible = users
                        .iter()
                        .filter(|user| user_matches(user, &search, &status))
                        .collect::<Vec<_>>();

                    if visible.is_empty() {
                        return view! {
                            <p class="cell-muted">"No users match this narrowing."</p>
                        }
                        .into_any();
                    }

                    view! {
                        <table class="data-table">
                            <thead>
                                <tr>
                                    <th>"Username"</th>
                                    <th>"Kind"</th>
                                    <th>"Status"</th>
                                    <th>"Email"</th>
                                    <th>"Name"</th>
                                    <th>"Created"</th>
                                    <th class="row-actions"></th>
                                </tr>
                            </thead>
                            <tbody>
                                {visible
                                    .into_iter()
                                    .map(|user| {
                                        let id = user.id.to_string();
                                        let detail_href = format!("/users/{id}");
                                        let edit_href = format!("/users/{id}/edit");
                                        let username = user.username.clone();
                                        let kind: &'static str = (&user.kind).into();
                                        let status: &'static str = (&user.status).into();
                                        let email = user.email.clone().unwrap_or_default();
                                        let name = full_name(user);
                                        let created = format_local(&user.created_at);

                                        view! {
                                            <tr>
                                                <td>{username.clone()}</td>
                                                <td class="cell-muted">{kind}</td>
                                                <td><span class="status">{status}</span></td>
                                                <td class="cell-muted">
                                                    {if email.is_empty() {
                                                        "—".to_string()
                                                    } else {
                                                        email
                                                    }}
                                                </td>
                                                <td>{name}</td>
                                                <td class="cell-muted">{created}</td>
                                                <td class="row-actions">
                                                    <A href=detail_href attr:class="btn">"Detail"</A>
                                                    {can_manage.get().then(|| {
                                                        view! {
                                                            <A href=edit_href attr:class="btn">"Edit"</A>
                                                        }
                                                    })}
                                                    {can_manage.get().then(|| {
                                                        view! {
                                                            <button
                                                                type="button"
                                                                class="btn btn-danger"
                                                                on:click=move |_| {
                                                                    handlers.request_delete.run((
                                                                        id.clone(),
                                                                        username.clone(),
                                                                    ));
                                                                }
                                                            >
                                                                "Delete"
                                                            </button>
                                                        }
                                                    })}
                                                </td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    }
                    .into_any()
                },
            })
        }}

        {move || {
            handlers.pending_delete.get().map(|pending| {
                let id = pending.id.clone();
                let message = format!(
                    "Delete \"{}\"? Their roles, permissions, and authorities go with them, and \
                     nothing is kept to restore.",
                    pending.name,
                );

                view! {
                    <div class="modal-overlay">
                        <div class="modal">
                            <h2 class="modal-title">"Delete user"</h2>
                            <p class="modal-message">{message}</p>

                            <div class="modal-actions">
                                <button
                                    type="button"
                                    on:click=move |_| {
                                        handlers.cancel_delete.run(());
                                    }
                                >
                                    "Cancel"
                                </button>
                                <button
                                    type="button"
                                    class="btn btn-danger"
                                    on:click=move |_| {
                                        handlers.confirm_delete.dispatch(id.clone());
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

// ---------------------------------------------------------- the new form

/// The new-user half of `/users/new`: every field is empty, `human` is the
/// kind on offer, and status starts `enabled` server-side.
#[component]
fn UserNewForm() -> impl IntoView {
    let state = expect_context::<AppState>();
    let handlers = handle_user_new_signals(state);
    let can_manage = can_manage(state);
    let create = handlers.create;

    let kind_select = NodeRef::<html::Select>::new();
    let username_input = NodeRef::<html::Input>::new();
    let email_input = NodeRef::<html::Input>::new();
    let first_name_input = NodeRef::<html::Input>::new();
    let last_name_input = NodeRef::<html::Input>::new();
    let profile_input = NodeRef::<html::Textarea>::new();

    let creating = create.pending();

    view! {
        <p class="back-link"><A href="/users">"Back to users"</A></p>

        <h1 class="page-title">"New user"</h1>

        <ErrorBanner errors=handlers.errors />

        <div class="form">
            <label>
                <span>"Kind"</span>
                <select node_ref=kind_select>
                    {USER_KINDS
                        .into_iter()
                        .map(|kind| view! { <option value=kind>{kind}</option> })
                        .collect_view()}
                </select>
            </label>

            <label>
                <span>"Username"</span>
                <input type="text" node_ref=username_input placeholder="required" />
            </label>

            <label>
                <span>"Email"</span>
                <input type="text" node_ref=email_input placeholder="optional" />
            </label>

            <label>
                <span>"First name"</span>
                <input type="text" node_ref=first_name_input placeholder="optional" />
            </label>

            <label>
                <span>"Last name"</span>
                <input type="text" node_ref=last_name_input placeholder="optional" />
            </label>

            <label>
                <span>"Profile (JSON)"</span>
                <textarea node_ref=profile_input placeholder="{}"></textarea>
            </label>

            <Show when=move || can_manage.get() fallback=|| view! { <NotPermitted /> }>
                <button
                    type="button"
                    class="btn btn-primary"
                    disabled=creating
                    on:click=move |_| {
                        create.dispatch(NewUserForm {
                            kind: kind_select.get().map(|el| el.value()).unwrap_or_default(),
                            username: username_input.get().map(|el| el.value()).unwrap_or_default(),
                            email: email_input.get().map(|el| el.value()).unwrap_or_default(),
                            first_name: first_name_input
                                .get()
                                .map(|el| el.value())
                                .unwrap_or_default(),
                            last_name: last_name_input.get().map(|el| el.value()).unwrap_or_default(),
                            profile: profile_input.get().map(|el| el.value()).unwrap_or_default(),
                        });
                    }
                >
                    {move || if creating.get() { "Creating..." } else { "Create" }}
                </button>
            </Show>
        </div>
    }
}

// ---------------------------------------------------------- the edit form

/// The edit half of `/users/:id/edit`. The form appears once the row has
/// loaded, seeded from it, and keeps the loaded row in its signals so the save
/// sends only what moved.
#[component]
fn UserEditForm(user_id: Signal<String>) -> impl IntoView {
    let state = expect_context::<AppState>();
    let handlers = handle_user_edit_signals(state, user_id);
    let can_manage = can_manage(state);
    let save = handlers.save;

    let username_input = NodeRef::<html::Input>::new();
    let email_input = NodeRef::<html::Input>::new();
    let first_name_input = NodeRef::<html::Input>::new();
    let last_name_input = NodeRef::<html::Input>::new();
    let status_select = NodeRef::<html::Select>::new();
    let profile_input = NodeRef::<html::Textarea>::new();

    let saving = save.pending();

    view! {
        <p class="back-link"><A href="/users">"Back to users"</A></p>

        <ErrorBanner errors=handlers.errors />

        {move || -> AnyView {
            handlers.user.with(|loaded| match loaded {
                LoadingState::Pending | LoadingState::Loading => {
                    view! { <p class="cell-muted">"Loading..."</p> }.into_any()
                },
                LoadingState::Error(err) => view! { <p class="error">{err.clone()}</p> }.into_any(),
                LoadingState::Loaded(user) => {
                    let kind: &'static str = (&user.kind).into();
                    let status: &'static str = (&user.status).into();
                    let username = user.username.clone();
                    let email = user.email.clone().unwrap_or_default();
                    let first_name = user.first_name.clone().unwrap_or_default();
                    let last_name = user.last_name.clone().unwrap_or_default();
                    let profile = profile_text(&user.profile);

                    view! {
                        <h1 class="page-title">"Edit user"</h1>

                        <div class="form">
                            <label>
                                <span>"Kind"</span>
                                // The api cannot move a user between kinds, so
                                // the loaded one is shown and never sent.
                                <select disabled=true>
                                    {USER_KINDS
                                        .into_iter()
                                        .map(|option| {
                                            view! {
                                                <option value=option selected=option == kind>
                                                    {option}
                                                </option>
                                            }
                                        })
                                        .collect_view()}
                                </select>
                            </label>

                            <label>
                                <span>"Username"</span>
                                <input
                                    type="text"
                                    node_ref=username_input
                                    prop:value=username
                                />
                            </label>

                            <label>
                                <span>"Email"</span>
                                <input type="text" node_ref=email_input prop:value=email />
                            </label>

                            <label>
                                <span>"First name"</span>
                                <input
                                    type="text"
                                    node_ref=first_name_input
                                    prop:value=first_name
                                />
                            </label>

                            <label>
                                <span>"Last name"</span>
                                <input
                                    type="text"
                                    node_ref=last_name_input
                                    prop:value=last_name
                                />
                            </label>

                            <label>
                                <span>"Status"</span>
                                <select node_ref=status_select>
                                    {USER_STATUSES
                                        .into_iter()
                                        .map(|option| {
                                            view! {
                                                <option value=option selected=option == status>
                                                    {option}
                                                </option>
                                            }
                                        })
                                        .collect_view()}
                                </select>
                            </label>

                            <label>
                                <span>"Profile (JSON)"</span>
                                <textarea node_ref=profile_input prop:value=profile></textarea>
                            </label>

                            <Show when=move || can_manage.get() fallback=|| view! { <NotPermitted /> }>
                                <button
                                    type="button"
                                    class="btn btn-primary"
                                    disabled=saving
                                    on:click=move |_| {
                                        save.dispatch(EditUserForm {
                                            id: user_id.get_untracked(),
                                            username: username_input
                                                .get()
                                                .map(|el| el.value())
                                                .unwrap_or_default(),
                                            email: email_input
                                                .get()
                                                .map(|el| el.value())
                                                .unwrap_or_default(),
                                            first_name: first_name_input
                                                .get()
                                                .map(|el| el.value())
                                                .unwrap_or_default(),
                                            last_name: last_name_input
                                                .get()
                                                .map(|el| el.value())
                                                .unwrap_or_default(),
                                            status: status_select
                                                .get()
                                                .map(|el| el.value())
                                                .unwrap_or_default(),
                                            profile: profile_input
                                                .get()
                                                .map(|el| el.value())
                                                .unwrap_or_default(),
                                        });
                                    }
                                >
                                    {move || if saving.get() { "Saving..." } else { "Save" }}
                                </button>
                            </Show>
                        </div>
                    }
                    .into_any()
                },
            })
        }}
    }
}

/// One route, two forms: `/users/new` carries no id, `/users/:id/edit` always
/// does, so the mode is settled once and the form that mounts stays mounted.
#[component]
pub fn UserFormPage() -> impl IntoView {
    let editing = use_id_param().get_untracked();

    if editing.trim().is_empty() {
        view! { <UserNewForm /> }.into_any()
    } else {
        let user_id = Signal::derive(move || editing.clone());

        view! { <UserEditForm user_id /> }.into_any()
    }
}

// ----------------------------------------------------------- the detail

/// `/users/:id`: the user as one card, then the three things a user can hold.
#[component]
pub fn UserDetailPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let user_id = use_id_param();
    let handlers = handle_user_detail_signals(state, user_id);
    let can_manage = can_manage(state);

    view! {
        <p class="back-link"><A href="/users">"Back to users"</A></p>

        <ErrorBanner errors=handlers.errors />

        {move || -> AnyView {
            handlers.user.with(|loaded| match loaded {
                LoadingState::Pending | LoadingState::Loading => {
                    view! { <p class="cell-muted">"Loading..."</p> }.into_any()
                },
                LoadingState::Error(err) => view! { <p class="error">{err.clone()}</p> }.into_any(),
                LoadingState::Loaded(user) => {
                    let id = user.id.to_string();
                    let edit_href = format!("/users/{id}/edit");
                    let username = user.username.clone();
                    let kind: &'static str = (&user.kind).into();
                    let status: &'static str = (&user.status).into();
                    let name = full_name(user);
                    let email = user.email.clone().unwrap_or_default();
                    let profile = profile_text(&user.profile);
                    let created = format_local(&user.created_at);
                    let updated = format_local(&user.updated_at);

                    view! {
                        <div class="page-header">
                            <h1 class="page-title">{username}</h1>

                            <div class="page-actions">
                                {can_manage.get().then(|| {
                                    view! {
                                        <A href=edit_href attr:class="button">"Edit"</A>
                                    }
                                })}
                            </div>
                        </div>

                        <div class="card">
                            <dl class="detail">
                                <dt>"Status"</dt>
                                <dd><span class="status">{status}</span></dd>

                                <dt>"Kind"</dt>
                                <dd>{kind}</dd>

                                <dt>"Name"</dt>
                                <dd>
                                    {if name.is_empty() {
                                        "—".to_string()
                                    } else {
                                        name
                                    }}
                                </dd>

                                <dt>"Email"</dt>
                                <dd>
                                    {if email.is_empty() {
                                        "—".to_string()
                                    } else {
                                        email
                                    }}
                                </dd>

                                <dt>"Id"</dt>
                                <dd class="record-id">{id}</dd>

                                <dt>"Created"</dt>
                                <dd>{created}</dd>

                                <dt>"Updated"</dt>
                                <dd>{updated}</dd>

                                <div class="detail-rule"></div>

                                <dt>"Profile"</dt>
                                <dd><pre class="detail-json">{profile}</pre></dd>
                            </dl>
                        </div>
                    }
                    .into_any()
                },
            })
        }}

        <UserRolesSection user_id=user_id />
        <UserPermissionsSection user_id=user_id />
        <UserAuthoritiesSection user_id=user_id />
    }
}

// ------------------------------------------------------- the grant sections
//
// One section each: a table of what the user holds, the control that grants
// one more, and a revoke per row. Every section refetches itself, so granting
// a role never refetches the permissions next door.

/// The roles a user holds, with the catalogue picker that grants another.
#[component]
fn UserRolesSection(user_id: Signal<String>) -> impl IntoView {
    let state = expect_context::<AppState>();
    let handlers = handle_user_roles_signals(state, user_id);
    let can_manage = can_manage(state);
    let (picked, set_picked) = signal(String::new());

    let grant = handlers.grant;
    let revoke = handlers.revoke;
    let refresh = handlers.refresh;

    let choices = Signal::derive(move || {
        handlers
            .catalogue
            .with(|catalogue| {
                match catalogue {
                    LoadingState::Pending | LoadingState::Loading => PickChoices::Loading,
                    LoadingState::Error(err) => {
                        PickChoices::Error(format!("roles unavailable: {err}"))
                    },
                    LoadingState::Loaded(roles) => {
                        let granted: Vec<String> = handlers
                            .grants
                            .with(|grants| {
                                match grants {
                                    LoadingState::Loaded(rows) => {
                                        rows.iter()
                                            .map(|row| row.grant.role_id.to_string())
                                            .collect()
                                    },
                                    _ => Vec::new(),
                                }
                            });

                        PickChoices::Ready(
                            roles
                                .iter()
                                .filter(|role| !granted.contains(&role.id.to_string()))
                                .map(|role| PickChoice::new(role.id.to_string(), role.name.clone()))
                                .collect(),
                        )
                    },
                }
            })
    });

    view! {
        <section class="section card">
            <h2 class="section-title">"Roles"</h2>

            <ErrorBanner errors=handlers.errors />

            <div class="create-row">
                <label>
                    <span>"Role"</span>
                    <PickCombo
                        choices=choices
                        placeholder="choose a role"
                        loading_text="loading roles..."
                        empty_text="every role is granted"
                        selected=picked.into()
                        on_pick=Callback::new(move |value: String| set_picked.set(value))
                    />
                </label>

                <Show when=move || can_manage.get() fallback=|| {}>
                    <button
                        type="button"
                        disabled=move || picked.get().is_empty()
                        on:click=move |_| {
                            grant.dispatch(picked.get());
                            set_picked.set(String::new());
                        }
                    >
                        "Grant"
                    </button>
                </Show>

                <button
                    type="button"
                    on:click=move |_| {
                        refresh.dispatch(());
                    }
                >
                    "Refresh"
                </button>
            </div>

            {move || -> AnyView {
                handlers.grants.with(|loaded| match loaded {
                    LoadingState::Pending | LoadingState::Loading => {
                        view! { <p class="cell-muted">"Loading..."</p> }.into_any()
                    },
                    LoadingState::Error(err) => view! { <p class="error">{err.clone()}</p> }.into_any(),
                    LoadingState::Loaded(rows) if rows.is_empty() => view! {
                        <p class="cell-muted">"No roles granted."</p>
                    }
                        .into_any(),
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
                                    .iter()
                                    .map(|row| {
                                        let role_id = row.grant.role_id.to_string();
                                        let name = row.role.name.clone();
                                        let granted = format_local(&row.grant.created_at);

                                        view! {
                                            <tr>
                                                <td>{name}</td>
                                                <td class="cell-muted">{granted}</td>
                                                <td class="row-actions">
                                                    {can_manage.get().then(|| {
                                                        view! {
                                                            <button
                                                                type="button"
                                                                class="btn btn-danger"
                                                                on:click=move |_| {
                                                                    revoke.dispatch(role_id.clone());
                                                                }
                                                            >
                                                                "Revoke"
                                                            </button>
                                                        }
                                                    })}
                                                </td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    }
                        .into_any(),
                })
            }}
        </section>
    }
}

/// The permissions granted to the user directly. The picker offers what the
/// catalogue holds and the user does not already hold — the api only grants
/// permissions that already exist, so a triple is never typed free-form.
#[component]
fn UserPermissionsSection(user_id: Signal<String>) -> impl IntoView {
    let state = expect_context::<AppState>();
    let handlers = handle_user_permissions_signals(state, user_id);
    let can_manage = can_manage(state);
    let (picked, set_picked) = signal(String::new());

    let grant = handlers.grant;
    let revoke = handlers.revoke;
    let refresh = handlers.refresh;

    let choices = Signal::derive(move || {
        handlers
            .catalogue
            .with(|catalogue| {
                match catalogue {
                    LoadingState::Pending | LoadingState::Loading => PickChoices::Loading,
                    LoadingState::Error(err) => {
                        PickChoices::Error(format!("permissions unavailable: {err}"))
                    },
                    LoadingState::Loaded(permissions) => {
                        let granted: Vec<String> = handlers
                            .grants
                            .with(|grants| {
                                match grants {
                                    LoadingState::Loaded(rows) => {
                                        rows.iter()
                                            .map(|row| row.permission.to_string())
                                            .collect()
                                    },
                                    _ => Vec::new(),
                                }
                            });

                        PickChoices::Ready(
                            permissions
                                .iter()
                                .filter(|permission| !granted.contains(&permission.to_string()))
                                .map(|permission| PickChoice::same(permission.to_string()))
                                .collect(),
                        )
                    },
                }
            })
    });

    view! {
        <section class="section card">
            <h2 class="section-title">"Permissions"</h2>

            <ErrorBanner errors=handlers.errors />

            <div class="create-row">
                <label>
                    <span>"Permission"</span>
                    <PickCombo
                        choices=choices
                        placeholder="choose a permission"
                        loading_text="loading permissions..."
                        empty_text="every permission is granted"
                        selected=picked.into()
                        on_pick=Callback::new(move |value: String| set_picked.set(value))
                    />
                </label>

                <Show when=move || can_manage.get() fallback=|| {}>
                    <button
                        type="button"
                        disabled=move || picked.get().is_empty()
                        on:click=move |_| {
                            grant.dispatch(picked.get());
                            set_picked.set(String::new());
                        }
                    >
                        "Grant"
                    </button>
                </Show>

                <button
                    type="button"
                    on:click=move |_| {
                        refresh.dispatch(());
                    }
                >
                    "Refresh"
                </button>
            </div>

            {move || -> AnyView {
                handlers.grants.with(|loaded| match loaded {
                    LoadingState::Pending | LoadingState::Loading => {
                        view! { <p class="cell-muted">"Loading..."</p> }.into_any()
                    },
                    LoadingState::Error(err) => view! { <p class="error">{err.clone()}</p> }.into_any(),
                    LoadingState::Loaded(rows) if rows.is_empty() => view! {
                        <p class="cell-muted">"No permissions granted."</p>
                    }
                        .into_any(),
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
                                    .iter()
                                    .map(|row| {
                                        let permission = row.permission.to_string();
                                        let granted = format_local(&row.grant.created_at);

                                        view! {
                                            <tr>
                                                <td>{permission.clone()}</td>
                                                <td class="cell-muted">{granted}</td>
                                                <td class="row-actions">
                                                    {can_manage.get().then(|| {
                                                        view! {
                                                            <button
                                                                type="button"
                                                                class="btn btn-danger"
                                                                on:click=move |_| {
                                                                    revoke.dispatch(permission.clone());
                                                                }
                                                            >
                                                                "Revoke"
                                                            </button>
                                                        }
                                                    })}
                                                </td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    }
                        .into_any(),
                })
            }}
        </section>
    }
}

/// The authorities — the login systems — the user has an identity in. The
/// params ride with the grant because each strategy reads its own: a
/// username/password authority wants `{"username": ..., "password": ...}`.
#[component]
fn UserAuthoritiesSection(user_id: Signal<String>) -> impl IntoView {
    let state = expect_context::<AppState>();
    let handlers = handle_user_authorities_signals(state, user_id);
    let can_manage = can_manage(state);
    let (picked, set_picked) = signal(String::new());
    let (params, set_params) = signal(String::new());

    let grant = handlers.grant;
    let revoke = handlers.revoke;
    let refresh = handlers.refresh;

    let choices = Signal::derive(move || {
        handlers
            .catalogue
            .with(|catalogue| {
                match catalogue {
                    LoadingState::Pending | LoadingState::Loading => PickChoices::Loading,
                    LoadingState::Error(err) => {
                        PickChoices::Error(format!("authorities unavailable: {err}"))
                    },
                    LoadingState::Loaded(authorities) => {
                        let granted: Vec<String> = handlers
                            .grants
                            .with(|grants| {
                                match grants {
                                    LoadingState::Loaded(rows) => {
                                        rows.iter()
                                            .map(|row| row.authority.id.to_string())
                                            .collect()
                                    },
                                    _ => Vec::new(),
                                }
                            });

                        PickChoices::Ready(
                            authorities
                                .iter()
                                .filter(|authority| !granted.contains(&authority.id.to_string()))
                                .map(|authority| {
                                    PickChoice::new(
                                        authority
                                            .client_key
                                            .to_string(),
                                        authority.name.clone(),
                                    )
                                })
                                .collect(),
                        )
                    },
                }
            })
    });

    view! {
        <section class="section card">
            <h2 class="section-title">"Authorities"</h2>

            <ErrorBanner errors=handlers.errors />

            <div class="create-row">
                <label>
                    <span>"Authority"</span>
                    <PickCombo
                        choices=choices
                        placeholder="choose an authority"
                        loading_text="loading authorities..."
                        empty_text="every authority is linked"
                        selected=picked.into()
                        on_pick=Callback::new(move |value: String| set_picked.set(value))
                    />
                </label>

                <label>
                    <span>"Params (JSON)"</span>
                    <input
                        type="text"
                        prop:value=move || params.get()
                        placeholder="{\"username\": ..., \"password\": ...}"
                        on:input=move |ev| set_params.set(event_target_value(&ev))
                    />
                </label>

                <Show when=move || can_manage.get() fallback=|| {}>
                    <button
                        type="button"
                        disabled=move || picked.get().is_empty()
                        on:click=move |_| {
                            grant.dispatch((picked.get(), params.get()));
                            set_picked.set(String::new());
                        }
                    >
                        "Grant"
                    </button>
                </Show>

                <button
                    type="button"
                    on:click=move |_| {
                        refresh.dispatch(());
                    }
                >
                    "Refresh"
                </button>
            </div>

            {move || -> AnyView {
                handlers.grants.with(|loaded| match loaded {
                    LoadingState::Pending | LoadingState::Loading => {
                        view! { <p class="cell-muted">"Loading..."</p> }.into_any()
                    },
                    LoadingState::Error(err) => view! { <p class="error">{err.clone()}</p> }.into_any(),
                    LoadingState::Loaded(rows) if rows.is_empty() => view! {
                        <p class="cell-muted">"No authorities linked."</p>
                    }
                        .into_any(),
                    LoadingState::Loaded(rows) => view! {
                        <table class="data-table">
                            <thead>
                                <tr>
                                    <th>"Authority"</th>
                                    <th>"Identifier"</th>
                                    <th>"Since"</th>
                                    <th class="row-actions"></th>
                                </tr>
                            </thead>
                            <tbody>
                                {rows
                                    .iter()
                                    .map(|row| {
                                        let authority_id = row.authority.id.to_string();
                                        let name = row.authority.name.clone();
                                        let identifier = row.user_authority.user_identifier.clone();
                                        let since = format_local(&row.user_authority.created_at);

                                        view! {
                                            <tr>
                                                <td>{name}</td>
                                                <td class="record-id">{identifier}</td>
                                                <td class="cell-muted">{since}</td>
                                                <td class="row-actions">
                                                    {can_manage.get().then(|| {
                                                        view! {
                                                            <button
                                                                type="button"
                                                                class="btn btn-danger"
                                                                on:click=move |_| {
                                                                    revoke.dispatch(authority_id.clone());
                                                                }
                                                            >
                                                                "Revoke"
                                                            </button>
                                                        }
                                                    })}
                                                </td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    }
                        .into_any(),
                })
            }}
        </section>
    }
}
