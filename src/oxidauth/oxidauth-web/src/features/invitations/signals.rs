use std::sync::Arc;

use chrono::{DateTime, NaiveDateTime, Utc};
use http::data::LoadingState;
use leptos::prelude::*;
use oxidauth::prelude::*;
use oxidauth_http::invitations::{
    create_invitation::CreateInvitationReq,
    delete_invitation::DeleteInvitationReq,
    find_invitation::FindInvitationReq,
};
use oxidauth_kernel::{
    invitations::{
        Invitation,
        create_invitation::CreateInvitationParams,
        delete_invitation::DeleteInvitationParams,
    },
    users::create_user::CreateUser,
};
use uuid::Uuid;

use crate::{
    features::auth::signals::{handle_feature_error, logout_callback},
    state::AppState,
};

/// `<input type="datetime-local">` yields `YYYY-MM-DDTHH:MM[:SS]` with no
/// zone; the DTO's `expires_at` is a `DateTime<Utc>` whose wire format is
/// RFC3339. The wall time is read as UTC — the form labels the field as
/// such — and re-serialized by serde into the exact RFC3339 shape.
pub(crate) fn parse_expiry(text: &str) -> Result<Option<DateTime<Utc>>, String> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }

    let naive = NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M"))
        .map_err(|err| format!("expiry must be a date and time: {err}"))?;

    Ok(Some(naive.and_utc()))
}

fn optional(text: &str) -> Option<String> {
    let text = text.trim().to_string();
    (!text.is_empty()).then_some(text)
}

// ---------------------------------------------------------------------------
// /invitations — lookup by id (the api has no listing).
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct HandleInvitationLookupResponse {
    pub errors: ReadSignal<Option<Vec<String>>>,
    /// Resolves with the found invitation's id; the page navigates on Ok.
    pub find: Action<String, Result<Uuid, String>>,
}

pub fn handle_invitation_lookup_signals(state: AppState) -> HandleInvitationLookupResponse {
    let (errors, set_errors) = signal(None::<Vec<String>>);
    let logout = logout_callback(state);

    let find = Action::new_unsync(move |id: &String| {
        let id = id.trim().to_string();
        let client = state.client.get_untracked();
        let logout = logout;

        set_errors.set(None);

        async move {
            match Uuid::parse_str(&id) {
                Ok(invitation_id) => {
                    match client
                        .find_invitation(FindInvitationReq { invitation_id })
                        .await
                    {
                        Ok(res) => Ok(res.invitation.id),
                        Err(err) => {
                            match handle_feature_error(err.as_ref(), &logout) {
                                Some(msg) => {
                                    set_errors.set(Some(vec![msg.clone()]));
                                    Err(msg)
                                },
                                // Session-dead: the logout redirect is the message.
                                None => Err(String::new()),
                            }
                        },
                    }
                },
                Err(_) => {
                    let msg = format!("invalid invitation id: {id}");
                    set_errors.set(Some(vec![msg.clone()]));
                    Err(msg)
                },
            }
        }
    });

    HandleInvitationLookupResponse { errors, find }
}

// ---------------------------------------------------------------------------
// /invitations/new — create (user registration payload + expiry).
// ---------------------------------------------------------------------------

/// Form fields as submitted; `expires_at` is the raw `datetime-local` text
/// and is parsed when the action runs.
#[derive(Clone, Debug, PartialEq)]
pub struct NewInvitationForm {
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub expires_at: String,
}

#[derive(Clone, Copy)]
pub struct HandleInvitationNewResponse {
    pub errors: ReadSignal<Option<Vec<String>>>,
    /// Resolves with the new invitation's id; the page navigates on Ok.
    pub create: Action<NewInvitationForm, Result<Uuid, String>>,
}

pub fn handle_invitation_new_signals(state: AppState) -> HandleInvitationNewResponse {
    let (errors, set_errors) = signal(None::<Vec<String>>);
    let logout = logout_callback(state);

    let store_error = Callback::new(move |msg: String| {
        set_errors.set(Some(vec![msg]));
    });

    let create = Action::new_unsync(move |form: &NewInvitationForm| {
        let form = form.clone();
        let client = state.client.get_untracked();
        let logout = logout;

        async move {
            let outcome: Result<Uuid, String> = async {
                let username = form
                    .username
                    .trim()
                    .to_string();
                if username.is_empty() {
                    return Err("username is required".to_string());
                }

                let expires_at = parse_expiry(&form.expires_at)?;

                let user = CreateUser {
                    id: None,
                    kind: None,
                    status: None,
                    username,
                    email: optional(&form.email),
                    first_name: optional(&form.first_name),
                    last_name: optional(&form.last_name),
                    profile: None,
                };

                let req = CreateInvitationReq {
                    invitation: CreateInvitationParams {
                        id: None,
                        expires_at,
                        user,
                    },
                };

                client
                    .create_invitation(req)
                    .await
                    .map(|res| res.invitation.id)
                    .map_err(|err| handle_feature_error(err.as_ref(), &logout).unwrap_or_default())
            }
            .await;

            if let Err(msg) = &outcome
                && !msg.is_empty()
            {
                store_error.run(msg.clone());
            }

            outcome
        }
    });

    HandleInvitationNewResponse { errors, create }
}

// ---------------------------------------------------------------------------
// /invitations/:id — find by id + delete.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct HandleInvitationDetailResponse {
    /// `Arc` because the kernel `Invitation` is not `Clone` and the signal
    /// must re-clone its value on every read.
    pub invitation: ReadSignal<LoadingState<Arc<Invitation>>>,
    pub errors: ReadSignal<Option<Vec<String>>>,
    /// Takes the id string; resolves `Ok(())` after the delete lands and the
    /// page navigates back to `/invitations`.
    pub delete: Action<String, Result<(), String>>,
}

pub fn handle_invitation_detail_signals(
    state: AppState,
    id: Signal<String>,
) -> HandleInvitationDetailResponse {
    let (invitation, set_invitation) = signal(LoadingState::<Arc<Invitation>>::Pending);
    let (errors, set_errors) = signal(None::<Vec<String>>);
    let logout = logout_callback(state);

    let fetch = Action::new_unsync(move |id: &String| {
        let id = id.clone();
        let client = state.client.get_untracked();
        let logout = logout;

        set_invitation.set(LoadingState::Loading);
        set_errors.set(None);

        async move {
            match Uuid::parse_str(&id) {
                Ok(invitation_id) => {
                    match client
                        .find_invitation(FindInvitationReq { invitation_id })
                        .await
                    {
                        Ok(res) => {
                            set_invitation.set(LoadingState::Loaded(Arc::new(res.invitation)))
                        },
                        Err(err) => {
                            if let Some(msg) = handle_feature_error(err.as_ref(), &logout) {
                                set_invitation.set(LoadingState::Error(msg));
                            }
                        },
                    }
                },
                Err(_) => {
                    set_invitation.set(LoadingState::Error(format!("invalid invitation id: {id}")))
                },
            };
        }
    });

    Effect::new(move |_| {
        fetch.dispatch(id.get());
    });

    let delete = Action::new_unsync(move |id: &String| {
        let id = id.clone();
        let client = state.client.get_untracked();
        let logout = logout;

        set_errors.set(None);

        async move {
            let outcome: Result<(), String> = match Uuid::parse_str(&id) {
                Ok(invitation_id) => {
                    client
                        .delete_invitation(DeleteInvitationReq {
                            invitation: DeleteInvitationParams { id: invitation_id },
                        })
                        .await
                        .map(|_| ())
                        .map_err(|err| {
                            handle_feature_error(err.as_ref(), &logout).unwrap_or_default()
                        })
                },
                Err(_) => Err(format!("invalid invitation id: {id}")),
            };

            if let Err(msg) = &outcome
                && !msg.is_empty()
            {
                set_errors.set(Some(vec![msg.clone()]));
            }

            outcome
        }
    });

    HandleInvitationDetailResponse {
        invitation,
        errors,
        delete,
    }
}
