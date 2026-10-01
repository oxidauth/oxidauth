use leptos::prelude::*;
use leptos_router::{NavigateOptions, components::A, hooks::use_navigate};
use oxidauth_kernel::authorities::DEFAULT_JWT_NBF;

use crate::{
    components::error_banner::ErrorBanner,
    features::authorities::signals::{
        AuthorityForm,
        AuthorityRow,
        HandleAuthoritiesListResponse,
        HandleAuthorityFormResponse,
        LoadingState,
        handle_authorities_list_signals,
        handle_authority_form_signals,
    },
    state::AppState,
};

/// The server-side challenge every authorities handler enforces; the console
/// hides the mutating affordances it would refuse.
const MANAGE: &str = "oxidauth:authorities:manage";

/// `/authorities` — every authority in one unpaginated fetch, narrowed
/// client-side by a name substring and a status select.
#[component]
pub fn AuthoritiesPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let handlers = handle_authorities_list_signals(state);
    let can_manage = state.can_gate(MANAGE);

    view! {
        <div class="page-header">
            <h1 class="page-title">"Authorities"</h1>
            {move || can_manage.get().then(|| view! {
                <div class="page-actions">
                    <A
                        href="/authorities/new"
                        attr:class="button new-link"
                        attr:aria-label="New authority"
                        attr:title="New authority"
                    >
                        "+"
                    </A>
                </div>
            })}
        </div>

        <div class="list-filter">
            <label>
                <span>"Name"</span>
                <input
                    type="search"
                    placeholder="filter by name"
                    prop:value=move || handlers.search.get()
                    on:input=move |ev| handlers.apply_search.run(event_target_value(&ev))
                />
            </label>
            <label>
                <span>"Status"</span>
                <select
                    prop:value=move || handlers.status.get()
                    on:change=move |ev| handlers.apply_status.run(event_target_value(&ev))
                >
                    <option value="">"all"</option>
                    <option value="enabled">"enabled"</option>
                    <option value="disabled">"disabled"</option>
                </select>
            </label>
        </div>

        <ErrorBanner errors=handlers.errors />

        {move || -> AnyView {
            handlers.authorities.with(|state| match state {
                LoadingState::Pending | LoadingState::Loading => {
                    view! { <p>"Loading..."</p> }.into_any()
                },
                LoadingState::Error(err) => view! { <p class="error">{err.clone()}</p> }.into_any(),
                LoadingState::Loaded(authorities) if authorities.is_empty() => {
                    view! { <p class="cell-muted">"No authorities yet."</p> }.into_any()
                },
                LoadingState::Loaded(_) => view! {
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>"Name"</th>
                                <th>"Strategy"</th>
                                <th>"Status"</th>
                                <th>"Client key"</th>
                                <th>"Created"</th>
                                <th class="row-actions"></th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                handlers
                                    .visible
                                    .get()
                                    .into_iter()
                                    .map(|row| {
                                        view! {
                                            <Row row handlers=handlers can_manage=can_manage />
                                        }
                                    })
                                    .collect_view()
                            }}
                        </tbody>
                    </table>

                    <Show
                        when=move || handlers.visible.get().is_empty()
                        fallback=move || {}
                    >
                        <p class="cell-muted">"No authorities match this filter."</p>
                    </Show>
                }
                .into_any(),
            })
        }}

        {move || {
            handlers.pending_delete.get().map(|pending| {
                let id = pending.id.clone();

                view! {
                    <div class="modal-overlay" on:click=move |_| handlers.cancel_delete.run(())>
                        <div class="modal" on:click=move |ev| ev.stop_propagation()>
                            <h2 class="modal-title">"Delete authority"</h2>
                            <p class="modal-message">{format!(
                                "Delete \"{}\"? Sign-in flows pointed at its client key stop working, and the authority cannot be restored.",
                                pending.name,
                            )}</p>
                            <div class="modal-actions">
                                <button
                                    class="btn"
                                    on:click=move |_| handlers.cancel_delete.run(())
                                >
                                    "Cancel"
                                </button>
                                <button
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

/// One table row. Clicking anywhere on it (or the Edit link) opens the edit
/// form; Delete asks first and never leaves the page.
#[component]
fn Row(
    row: AuthorityRow,
    handlers: HandleAuthoritiesListResponse,
    can_manage: Signal<bool>,
) -> impl IntoView {
    let href = format!("/authorities/{}/edit", row.id);
    let navigate = use_navigate();
    let href_on_click = href.clone();
    let id_for_delete = row.id.clone();
    let name_for_delete = row.name.clone();

    view! {
        <tr
            class="row-link"
            on:click=move |_| navigate(&href_on_click, NavigateOptions::default())
        >
            <td>{row.name.clone()}</td>
            <td>{row.strategy.clone()}</td>
            <td>{row.status.clone()}</td>
            <td class="record-id cell-muted">{row.client_key.clone()}</td>
            <td class="cell-muted">{row.created.clone()}</td>
            <td class="row-actions">
                <A href=href attr:class="btn">"Edit"</A>
                {move || {
                    let id_for_delete = id_for_delete.clone();
                    let name_for_delete = name_for_delete.clone();

                    can_manage.get().then(move || view! {
                        <button
                            class="btn btn-danger"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                handlers
                                    .request_delete
                                    .run((id_for_delete.clone(), name_for_delete.clone()));
                            }
                        >
                            "Delete"
                        </button>
                    })
                }}
            </td>
        </tr>
    }
}

/// `/authorities/new` and `/authorities/:id/edit` — one form, seeded from
/// `find_authority_by_id` on the edit route and empty on the new one.
#[component]
pub fn AuthorityFormPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let handlers = handle_authority_form_signals(state);

    let save = handlers.save;
    let navigate = use_navigate();

    Effect::new(move |_| {
        if let Some(Ok(_)) = save.value().get() {
            navigate("/authorities", NavigateOptions::default());
        }
    });

    let can_manage = state.can_gate(MANAGE);

    // the edit form waits for the preloaded authority; creation starts blank
    let fields_ready = Signal::derive(move || {
        if !handlers.is_edit.get() {
            true
        } else {
            handlers
                .authority
                .with(|state| matches!(state, LoadingState::Loaded(_)))
        }
    });

    view! {
        <p class="back-link"><A href="/authorities">"Back to authorities"</A></p>

        <div class="page-header">
            <h1 class="page-title">
                {move || if handlers.is_edit.get() { "Edit authority" } else { "New authority" }}
            </h1>
        </div>

        <ErrorBanner errors=handlers.errors />

        <Show
            when=move || fields_ready.get()
            fallback=move || view! {
                {move || -> AnyView {
                    handlers.authority.with(|state| match state {
                        LoadingState::Error(err) => {
                            view! { <p class="error">{err.clone()}</p> }.into_any()
                        },
                        _ => view! { <p>"Loading..."</p> }.into_any(),
                    })
                }}
            }
        >
            <AuthorityFormView handlers=handlers can_manage=can_manage />
        </Show>
    }
}

#[component]
fn AuthorityFormView(
    handlers: HandleAuthorityFormResponse,
    can_manage: Signal<bool>,
) -> impl IntoView {
    let fields = handlers.fields;
    let save = handlers.save;
    let is_edit = handlers.is_edit.get();
    let authority_id = handlers.authority_id;

    let nbf = fields.nbf;
    let totp = fields.totp;

    view! {
        <form
            class="form"
            on:submit=move |ev| {
                ev.prevent_default();

                let id = authority_id.get();
                let client_key = fields.client_key.get();

                save.dispatch(AuthorityForm {
                    id: (!id.is_empty()).then_some(id),
                    // the server rotates an omitted client key, so the edit
                    // form carries the loaded one back verbatim
                    client_key: is_edit.then_some(client_key).filter(|key| !key.is_empty()),
                    name: fields.name.get(),
                    strategy: fields.strategy.get(),
                    status: fields.status.get(),
                    jwt_ttl: fields.jwt_ttl.get(),
                    refresh_token_ttl: fields.refresh_token_ttl.get(),
                    nbf: fields.nbf.get(),
                    nbf_offset: fields.nbf_offset.get(),
                    entitlements_encoding: fields.entitlements_encoding.get(),
                    totp: fields.totp.get(),
                    totp_ttl: fields.totp_ttl.get(),
                    webhook: fields.webhook.get(),
                    webhook_key: fields.webhook_key.get(),
                    params: fields.params.get(),
                });
            }
        >
            <label class="form-label">
                <span>"Name"</span>
                <input
                    class="form-input"
                    type="text"
                    prop:value=move || fields.name.get()
                    on:input=move |ev| fields.name.set(event_target_value(&ev))
                />
            </label>

            <label class="form-label">
                <span>"Strategy"</span>
                <select
                    prop:value=move || fields.strategy.get()
                    on:change=move |ev| fields.strategy.set(event_target_value(&ev))
                >
                    <option value="username_password">"username_password"</option>
                    <option value="single_use_token">"single_use_token"</option>
                    <option value="oauth2">"oauth2"</option>
                </select>
            </label>

            <label class="form-label">
                <span>"Status"</span>
                <select
                    prop:value=move || fields.status.get()
                    on:change=move |ev| fields.status.set(event_target_value(&ev))
                >
                    <option value="enabled">"enabled"</option>
                    <option value="disabled">"disabled"</option>
                </select>
            </label>

            <label class="form-label">
                <span>"Jwt ttl (seconds)"</span>
                <input
                    class="form-input"
                    type="text"
                    prop:value=move || fields.jwt_ttl.get()
                    on:input=move |ev| fields.jwt_ttl.set(event_target_value(&ev))
                />
            </label>

            <label class="form-label">
                <span>"Refresh token ttl (seconds)"</span>
                <input
                    class="form-input"
                    type="text"
                    prop:value=move || fields.refresh_token_ttl.get()
                    on:input=move |ev| fields.refresh_token_ttl.set(event_target_value(&ev))
                />
            </label>

            <label class="form-label">
                <span>"Jwt nbf offset"</span>
                <select
                    prop:value=move || fields.nbf.get()
                    on:change=move |ev| fields.nbf.set(event_target_value(&ev))
                >
                    <option value="disabled">"disabled"</option>
                    <option value="enabled">"enabled"</option>
                </select>
            </label>

            <Show
                when=move || nbf.get() == "enabled"
                fallback=move || {}
            >
                <label class="form-label">
                    <span>{format!("Nbf offset (seconds, default {})", DEFAULT_JWT_NBF.as_secs())}</span>
                    <input
                        class="form-input"
                        type="text"
                        prop:value=move || fields.nbf_offset.get()
                        on:input=move |ev| fields.nbf_offset.set(event_target_value(&ev))
                    />
                </label>
            </Show>

            <label class="form-label">
                <span>"Entitlements encoding"</span>
                <select
                    prop:value=move || fields.entitlements_encoding.get()
                    on:change=move |ev| fields.entitlements_encoding.set(event_target_value(&ev))
                >
                    <option value="txt">"txt"</option>
                    <option value="gz">"gz"</option>
                </select>
            </label>

            <label class="form-label">
                <span>"Totp"</span>
                <select
                    prop:value=move || fields.totp.get()
                    on:change=move |ev| fields.totp.set(event_target_value(&ev))
                >
                    <option value="disabled">"disabled"</option>
                    <option value="enabled">"enabled"</option>
                </select>
            </label>

            <Show
                when=move || totp.get() == "enabled"
                fallback=move || {}
            >
                <label class="form-label">
                    <span>"Totp ttl (seconds)"</span>
                    <input
                        class="form-input"
                        type="text"
                        prop:value=move || fields.totp_ttl.get()
                        on:input=move |ev| fields.totp_ttl.set(event_target_value(&ev))
                    />
                </label>

                <label class="form-label">
                    <span>"Totp webhook url"</span>
                    <input
                        class="form-input"
                        type="text"
                        placeholder="https://host/totp"
                        prop:value=move || fields.webhook.get()
                        on:input=move |ev| fields.webhook.set(event_target_value(&ev))
                    />
                </label>

                <label class="form-label">
                    <span>"Totp webhook key"</span>
                    <input
                        class="form-input"
                        type="text"
                        prop:value=move || fields.webhook_key.get()
                        on:input=move |ev| fields.webhook_key.set(event_target_value(&ev))
                    />
                </label>
            </Show>

            <label class="form-label">
                <span>"Params (JSON)"</span>
                <textarea
                    class="form-input"
                    rows=8
                    placeholder="{}"
                    prop:value=move || fields.params.get()
                    on:input=move |ev| fields.params.set(event_target_value(&ev))
                >
                </textarea>
            </label>

            {move || can_manage.get().then(|| view! {
                <button
                    class="btn btn-primary"
                    type="submit"
                    disabled=move || save.pending().get()
                >
                    {move || {
                        if save.pending().get() {
                            "Saving..."
                        } else if is_edit {
                            "Save changes"
                        } else {
                            "Create authority"
                        }
                    }}
                </button>
            })}
        </form>
    }
}
