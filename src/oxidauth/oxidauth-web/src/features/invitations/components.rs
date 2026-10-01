use chrono::Utc;
use http::data::LoadingState;
use leptos::prelude::*;
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_params},
    params::Params,
};
use serde::Deserialize;

use crate::{
    components::error_banner::ErrorBanner,
    features::invitations::signals::{
        HandleInvitationDetailResponse,
        HandleInvitationLookupResponse,
        HandleInvitationNewResponse,
        NewInvitationForm,
        handle_invitation_detail_signals,
        handle_invitation_lookup_signals,
        handle_invitation_new_signals,
    },
    state::AppState,
    time::format_local,
};

/// `/invitations/:id` path parameter.
#[derive(Clone, Debug, PartialEq, Deserialize, Params)]
pub struct InvitationIdParams {
    pub id: String,
}

fn use_id_param() -> Signal<String> {
    let params = use_params::<InvitationIdParams>();

    Signal::derive(move || {
        params
            .get()
            .map(|params| params.id)
            .unwrap_or_default()
    })
}

/// There is deliberately no table here: the api exposes no invitation
/// listing, so the page is a lookup-by-id card plus the create entry point.
#[component]
pub fn InvitationsPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let HandleInvitationLookupResponse { errors, find } = handle_invitation_lookup_signals(state);

    let (lookup_id, set_lookup_id) = signal(String::new());
    let navigate = use_navigate();

    Effect::new(move |_| {
        if let Some(Ok(invitation_id)) = find.value().get() {
            navigate(&format!("/invitations/{invitation_id}"), Default::default());
        }
    });

    let can_read = state.can_gate("oxidauth:invitations:read");
    let can_create = state.can_gate("oxidauth:invitations:create");
    let submitting = find.pending();

    view! {
        <div class="page-header">
            <h1 class="page-title">"Invitations"</h1>

            <div class="page-actions">
                {move || can_create.get().then(|| {
                    view! {
                        <A
                            href="/invitations/new"
                            attr:class="button plus-action"
                            attr:aria-label="New invitation"
                            attr:title="New invitation"
                        >
                            "+"
                        </A>
                    }
                })}
            </div>
        </div>

        <ErrorBanner errors />

        <div class="card">
            <form
                class="form"
                on:submit=move |ev| {
                    ev.prevent_default();
                    find.dispatch(lookup_id.get());
                }
            >
                <label class="form-label">
                    <span>"Invitation id"</span>
                    <input
                        class="form-input"
                        type="text"
                        placeholder="invitation uuid"
                        prop:value=move || lookup_id.get()
                        on:input=move |ev| set_lookup_id.set(event_target_value(&ev))
                    />
                </label>

                {move || can_read.get().then(|| {
                    view! {
                        <button
                            class="btn btn-primary"
                            type="submit"
                            disabled=move || submitting.get()
                        >
                            {move || if submitting.get() { "Finding..." } else { "Find" }}
                        </button>
                    }
                })}
            </form>
        </div>

        <p class="cell-muted">
            "The api does not support listing invitations — look one up by its id."
        </p>
    }
}

#[component]
pub fn InvitationFormPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let HandleInvitationNewResponse { errors, create } = handle_invitation_new_signals(state);

    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (first_name, set_first_name) = signal(String::new());
    let (last_name, set_last_name) = signal(String::new());
    let (expires_at, set_expires_at) = signal(String::new());

    let navigate = use_navigate();

    Effect::new(move |_| {
        if let Some(Ok(invitation_id)) = create.value().get() {
            navigate(&format!("/invitations/{invitation_id}"), Default::default());
        }
    });

    let can_create = state.can_gate("oxidauth:invitations:create");
    let submitting = create.pending();

    let field = move |label: &'static str,
                      value: ReadSignal<String>,
                      set_value: WriteSignal<String>,
                      input_type: &'static str,
                      placeholder: &'static str|
          -> AnyView {
        view! {
            <label class="form-label">
                <span>{label}</span>
                <input
                    class="form-input"
                    type=input_type
                    placeholder=placeholder
                    prop:value=move || value.get()
                    on:input=move |ev| set_value.set(event_target_value(&ev))
                />
            </label>
        }
        .into_any()
    };

    view! {
        <p class="back-link"><A href="/invitations">"Back to invitations"</A></p>

        <div class="page-header">
            <h1 class="page-title">"New invitation"</h1>
        </div>

        <ErrorBanner errors />

        <form
            class="form"
            on:submit=move |ev| {
                ev.prevent_default();
                create.dispatch(NewInvitationForm {
                    username: username.get(),
                    email: email.get(),
                    first_name: first_name.get(),
                    last_name: last_name.get(),
                    expires_at: expires_at.get(),
                });
            }
        >
            {field("Username", username, set_username, "text", "required")}
            {field("Email", email, set_email, "email", "optional")}
            {field("First name", first_name, set_first_name, "text", "optional")}
            {field("Last name", last_name, set_last_name, "text", "optional")}
            {field(
                "Expires at (UTC)",
                expires_at,
                set_expires_at,
                "datetime-local",
                "",
            )}

            {move || can_create.get().then(|| {
                view! {
                    <button
                        class="btn btn-primary"
                        type="submit"
                        disabled=move || submitting.get()
                    >
                        {move || if submitting.get() { "Creating..." } else { "Create" }}
                    </button>
                }
            })}
        </form>
    }
}

#[component]
pub fn InvitationDetailPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let id = use_id_param();
    let handlers: HandleInvitationDetailResponse = handle_invitation_detail_signals(state, id);

    let (show_delete, set_show_delete) = signal(false);
    let navigate = use_navigate();

    Effect::new(move |_| {
        if let Some(Ok(())) = handlers.delete.value().get() {
            navigate("/invitations", Default::default());
        }
    });

    let delete_id = id;
    let can_delete = state.can_gate("oxidauth:invitations:delete");
    let deleting = handlers.delete.pending();

    view! {
        <p class="back-link"><A href="/invitations">"Back to invitations"</A></p>

        <div class="page-header">
            <h1 class="page-title">"Invitation"</h1>

            <div class="page-actions">
                {move || can_delete.get().then(|| {
                    view! {
                        <button
                            class="btn btn-danger"
                            type="button"
                            on:click=move |_| set_show_delete.set(true)
                        >
                            "Delete"
                        </button>
                    }
                })}
            </div>
        </div>

        <ErrorBanner errors=handlers.errors />

        {move || -> AnyView {
            match handlers.invitation.get() {
                LoadingState::Pending | LoadingState::Loading => {
                    view! { <p class="cell-muted">"Loading..."</p> }.into_any()
                },
                LoadingState::Error(err) => view! { <p class="form-error">{err}</p> }.into_any(),
                LoadingState::Loaded(invitation) => view! {
                    <div class="card">
                        <dl class="detail">
                            <dt>"Invitation"</dt>
                            <dd class="record-id">{invitation.id.to_string()}</dd>

                            <dt>"User"</dt>
                            <dd class="record-id">{invitation.user_id.to_string()}</dd>

                            <dt>"Expires"</dt>
                            <dd>
                                {format_local(&invitation.expires_at)}
                                {if invitation.expires_at < Utc::now() {
                                    view! { <span class="form-error">" (expired)"</span> }.into_any()
                                } else {
                                    ().into_any()
                                }}
                            </dd>

                            <dt>"Created"</dt>
                            <dd>{format_local(&invitation.created_at)}</dd>

                            <dt>"Updated"</dt>
                            <dd>{format_local(&invitation.updated_at)}</dd>
                        </dl>
                    </div>
                }
                .into_any(),
            }
        }}

        <Show
            when=move || show_delete.get()
            fallback=move || {}
        >
            <div class="modal-overlay" on:click=move |_| set_show_delete.set(false)>
                <div class="modal" on:click=move |ev| ev.stop_propagation()>
                    <h2 class="modal-title">"Delete invitation?"</h2>
                    <p class="modal-message">
                        "The invitee will no longer be able to accept this invitation. This cannot be undone."
                    </p>
                    <div class="modal-actions">
                        <button
                            type="button"
                            class="btn"
                            on:click=move |_| set_show_delete.set(false)
                        >
                            "Cancel"
                        </button>
                        <button
                            type="button"
                            class="btn btn-danger"
                            disabled=move || deleting.get()
                            on:click=move |_| {
                                handlers.delete.dispatch(delete_id.get());
                            }
                        >
                            "Delete"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
