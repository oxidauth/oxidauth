//! Signals and actions for the users slice: the list, the create/edit forms,
//! the detail card, and the three grant sub-resources a user carries (roles,
//! permissions, authorities).
//!
//! Every users endpoint is unpaginated server-side, so the list keeps the
//! whole set and the browser narrows it. Components stay view-only glue: the
//! actions take plain strings (ids, permission text, json text) and own the
//! parsing, the api call, the error reporting, and the refetch.

pub use http::data::LoadingState;
use leptos::{logging::log, prelude::*};
use leptos_router::hooks::use_navigate;
/// `UpdateUserTrait` is not in the sdk prelude, and the role grant takes the
/// wrapper's own param struct, so both are named where they are called.
use oxidauth::client::users::{
    roles::create_user_role::CreateUserRole,
    update_user::UpdateUserTrait,
};
use oxidauth::prelude::*;
use oxidauth_http::users::{
    authorities::create_user_authority::{CreateUserAuthorityBodyReq, UserAuthorityParams},
    create_user::CreateUserReq,
    update_user::{UpdateUserBodyReq, UpdateUserUser},
};
use oxidauth_kernel::{
    JsonValue,
    authorities::{Authority, list_all_authorities::ListAllAuthorities},
    permissions::{Permission, list_all_permissions::ListAllPermissions},
    roles::{Role, list_all_roles::ListAllRoles},
    user_authorities::{
        UserAuthorityWithAuthority,
        delete_user_authority::DeleteUserAuthority,
        list_user_authorities_by_user_id::ListUserAuthoritiesByUserId,
    },
    user_permission_grants::{
        UserPermission,
        create_user_permission_grant::CreateUserPermission,
        delete_user_permission_grant::DeleteUserPermission,
        list_user_permission_grants_by_user_id::ListUserPermissionGrantsByUserId,
    },
    user_role_grants::UserRole,
    users::{
        User,
        UserKind,
        UserStatus,
        create_user::CreateUser,
        find_user_by_id::FindUserById,
        list_all_users::ListAllUsers,
    },
};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    features::auth::signals::{handle_feature_error, logout_callback},
    state::AppState,
};

/// What the api enforces on every users route, and what this slice gates its
/// mutating buttons on.
pub const USERS_MANAGE: &str = "oxidauth:users:manage";

/// The `UserKind` select, in the serde spelling the api accepts.
pub const USER_KINDS: [&str; 2] = ["human", "api"];

/// The `UserStatus` selects (list filter, edit form), in the serde spelling.
pub const USER_STATUSES: [&str; 3] = ["enabled", "invited", "disabled"];

/// The one place a route's text id becomes a call's uuid.
fn user_uuid(id: &str) -> Result<Uuid, String> {
    Uuid::parse_str(id.trim()).map_err(|_| format!("invalid user id: {id}"))
}

/// The user this page edits, resolved before the call so a malformed route
/// never reaches the api.
fn resolve_user_id(
    user_id: Signal<String>,
    set_errors: WriteSignal<Option<Vec<String>>>,
) -> Option<Uuid> {
    match user_uuid(&user_id.get_untracked()) {
        Ok(user_id) => Some(user_id),
        Err(message) => {
            set_errors.set(Some(vec![message]));

            None
        },
    }
}

/// Reports a failed mutation: a dead session ends the session and is
/// swallowed (the redirect is the message), anything else becomes the page's
/// own error text.
fn report(err: BoxedError, logout: &Callback<()>, set_errors: WriteSignal<Option<Vec<String>>>) {
    if let Some(message) = handle_feature_error(err.as_ref(), logout) {
        log!("users: {message}");
        set_errors.set(Some(vec![message]));
    }
}

/// The same verdict for a load, which also has to empty the thing it was
/// filling — a list that failed to load shows the failure, not the rows of
/// whatever was there before.
fn report_load(
    err: BoxedError,
    logout: &Callback<()>,
    set_errors: WriteSignal<Option<Vec<String>>>,
    set_failed: impl FnOnce(String),
) {
    if let Some(message) = handle_feature_error(err.as_ref(), logout) {
        log!("users: {message}");
        set_errors.set(Some(vec![message.clone()]));
        set_failed(message);
    }
}

/// The profile as the form textarea and the detail `<pre>` both show it.
pub fn profile_text(profile: &Value) -> String {
    serde_json::to_string_pretty(profile).unwrap_or_else(|_| profile.to_string())
}

/// The names as one cell: `Ada Lovelace`, or nothing when neither half is
/// stored.
pub fn full_name(user: &User) -> String {
    [user.first_name.as_deref(), user.last_name.as_deref()]
        .into_iter()
        .flatten()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The list's browser-side narrowing: free text over username, email and both
/// names, plus one status. The empty string is "no narrowing".
pub fn user_matches(user: &User, search: &str, status: &str) -> bool {
    if !status.is_empty() && <&'static str>::from(&user.status) != status {
        return false;
    }

    let needle = search.trim().to_lowercase();

    needle.is_empty()
        || [
            Some(user.username.as_str()),
            user.email.as_deref(),
            user.first_name.as_deref(),
            user.last_name.as_deref(),
        ]
        .into_iter()
        .flatten()
        .any(|field| {
            field
                .to_lowercase()
                .contains(&needle)
        })
}

// ---------------------------------------------------------------- list

/// The row a pending delete confirmation refers to.
#[derive(Clone, Debug, PartialEq)]
pub struct PendingDelete {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Copy)]
pub struct HandleUsersListResponse {
    pub users: ReadSignal<LoadingState<Vec<User>>>,
    /// The narrowing in force, applied by [`user_matches`] over the loaded
    /// set — the api has no filter of its own.
    pub search: ReadSignal<String>,
    pub status: ReadSignal<String>,
    pub errors: ReadSignal<Option<Vec<String>>>,
    pub pending_delete: ReadSignal<Option<PendingDelete>>,
    pub apply_search: Callback<String>,
    pub apply_status: Callback<String>,
    pub request_delete: Callback<(String, String)>,
    pub cancel_delete: Callback<()>,
    pub confirm_delete: Action<String, ()>,
}

pub fn handle_users_list_signals(state: AppState) -> HandleUsersListResponse {
    let logout = logout_callback(state);

    let (users, set_users) = signal(LoadingState::<Vec<User>>::Pending);
    let (search, set_search) = signal(String::new());
    let (status, set_status) = signal(String::new());
    let (errors, set_errors) = signal(None::<Vec<String>>);
    let (pending_delete, set_pending_delete) = signal(None::<PendingDelete>);

    let fetch = Action::new_unsync(move |_: &()| {
        let client = state.client.get_untracked();

        set_users.set(LoadingState::Loading);
        set_errors.set(None);

        async move {
            match client
                .list_all_users(ListAllUsers)
                .await
            {
                Ok(res) => set_users.set(LoadingState::Loaded(res.users)),
                Err(err) => {
                    report_load(err, &logout, set_errors, |message| {
                        set_users.set(LoadingState::Error(message));
                    })
                },
            }
        }
    });

    Effect::new(move |_| {
        fetch.dispatch(());
    });

    // Client-side narrowing moves the table only, never the fetch.
    let apply_search = Callback::new(move |text: String| {
        set_search.set(text);
    });

    let apply_status = Callback::new(move |value: String| {
        set_status.set(value);
    });

    let request_delete = Callback::new(move |(id, name): (String, String)| {
        set_pending_delete.set(Some(PendingDelete { id, name }));
    });

    let cancel_delete = Callback::new(move |_: ()| {
        set_pending_delete.set(None);
    });

    let confirm_delete = Action::new_unsync(move |id: &String| {
        let id = id.clone();
        let client = state.client.get_untracked();

        set_errors.set(None);
        set_pending_delete.set(None);

        async move {
            match user_uuid(&id) {
                Ok(user_id) => {
                    match client
                        .delete_user(user_id)
                        .await
                    {
                        Ok(_) => {
                            fetch.dispatch(());
                        },
                        Err(err) => report(err, &logout, set_errors),
                    }
                },
                Err(message) => set_errors.set(Some(vec![message])),
            }
        }
    });

    HandleUsersListResponse {
        users,
        search,
        status,
        errors,
        pending_delete,
        apply_search,
        apply_status,
        request_delete,
        cancel_delete,
        confirm_delete,
    }
}

// ------------------------------------------------------------- the form

/// An absent form field is no value at all, so nothing is invented here.
fn optional_text(text: &str) -> Option<String> {
    let text = text.trim();

    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

/// The profile box: an empty box stores nothing, and a stored profile is an
/// object — the shape the column defaults to.
fn parse_profile(text: &str) -> Result<Option<Value>, String> {
    if text.trim().is_empty() {
        return Ok(None);
    }

    let value: Value = serde_json::from_str(text.trim())
        .map_err(|err| format!("profile must be valid json: {err}"))?;

    if !value.is_object() {
        return Err("profile must be a json object, e.g. {}".to_string());
    }

    Ok(Some(value))
}

/// The creation form as it submits: raw text, parsed when the action runs.
/// Status has no box here — it starts `enabled` server-side.
#[derive(Clone, Debug, PartialEq)]
pub struct NewUserForm {
    pub kind: String,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub profile: String,
}

#[derive(Clone, Copy)]
pub struct HandleUserNewResponse {
    pub errors: ReadSignal<Option<Vec<String>>>,
    pub create: Action<NewUserForm, ()>,
}

pub fn handle_user_new_signals(state: AppState) -> HandleUserNewResponse {
    let logout = logout_callback(state);
    let navigate = use_navigate();

    let (errors, set_errors) = signal(None::<Vec<String>>);

    let create = Action::new_unsync(move |form: &NewUserForm| {
        let form = form.clone();
        let client = state.client.get_untracked();
        let navigate = navigate.clone();

        set_errors.set(None);

        async move {
            let request: Result<CreateUserReq, String> = (|| {
                let username = optional_text(&form.username)
                    .ok_or_else(|| "username is required".to_string())?;

                let kind: UserKind = form
                    .kind
                    .trim()
                    .parse()
                    .map_err(|err| format!("unknown user kind: {err}"))?;

                Ok(CreateUserReq {
                    user: CreateUser {
                        id: None,
                        kind: Some(kind),
                        status: None,
                        username,
                        email: optional_text(&form.email),
                        first_name: optional_text(&form.first_name),
                        last_name: optional_text(&form.last_name),
                        profile: parse_profile(&form.profile)?,
                    },
                })
            })();

            let request = match request {
                Ok(request) => request,
                Err(message) => {
                    set_errors.set(Some(vec![message]));

                    return;
                },
            };

            match client
                .create_user(request)
                .await
            {
                Ok(res) => navigate(&format!("/users/{}", res.user.id), Default::default()),
                Err(err) => report(err, &logout, set_errors),
            }
        }
    });

    HandleUserNewResponse { errors, create }
}

/// The row the edit form was seeded from, in the shape the submit compares
/// against.
#[derive(Clone, Debug)]
struct LoadedUser {
    username: String,
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    status: UserStatus,
    profile: String,
}

/// The edit form as it submits. Everything arrives as text — the selects
/// carry serde tokens, the profile arrives raw — so the component parses
/// nothing.
#[derive(Clone, Debug, PartialEq)]
pub struct EditUserForm {
    pub id: String,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub status: String,
    pub profile: String,
}

#[derive(Clone, Copy)]
pub struct HandleUserEditResponse {
    pub user: ReadSignal<LoadingState<User>>,
    pub errors: ReadSignal<Option<Vec<String>>>,
    pub save: Action<EditUserForm, ()>,
}

/// The value one text field carries in the update body.
///
/// `PUT /users/{id}` is a full-row replace whose body cannot tell an omitted
/// key from a null one: the dto serializes all six fields and serde reads both
/// spellings as `None` (OXA-000015). What a null then means differs by column
/// — the service backfills `username` and `email` from the stored row, so
/// there a null *keeps* the value, while the two names pass straight through,
/// so there a null *erases* one. Hence:
///
/// - a field the form did not move carries null for `username`/`email` (keep) and its stored text
///   for a name (keep) — `resend_when_unchanged` picks between the two;
/// - a field the form moved carries its new text, or null when the box was emptied. Only the
///   nullable names take that clear; the caller refuses the rest rather than sending a null that
///   changes nothing.
fn wire_text(loaded: Option<&str>, edited: &str, resend_when_unchanged: bool) -> Option<String> {
    let edited = edited.trim();

    match loaded {
        Some(stored) if stored == edited => {
            if resend_when_unchanged {
                (!stored.is_empty()).then(|| stored.to_string())
            } else {
                None
            }
        },
        // Moved, and emptied: the null is the clear.
        _ if edited.is_empty() => None,
        _ => Some(edited.to_string()),
    }
}

/// The profile's verdict, with the one case plain text cannot carry: an
/// emptied box means an empty document, because omitting the key would keep
/// the stored one.
fn diff_profile(loaded: &str, edited: &str) -> Result<Option<Value>, String> {
    let edited = edited.trim();

    if edited == loaded.trim() {
        return Ok(None);
    }

    if edited.is_empty() {
        return Ok(Some(Value::Object(serde_json::Map::new())));
    }

    parse_profile(edited)
}

/// The body of one `PUT`: only what the form actually moved, plus the two
/// names the api would otherwise null.
fn build_update_body(
    form: &EditUserForm,
    loaded: &LoadedUser,
) -> Result<UpdateUserBodyReq, Vec<String>> {
    let mut messages = Vec::new();

    let username = wire_text(Some(&loaded.username), &form.username, false);

    if form
        .username
        .trim()
        .is_empty()
    {
        messages.push("username is required".to_string());
    }

    let email = wire_text(loaded.email.as_deref(), &form.email, false);

    if loaded.email.is_some() && form.email.trim().is_empty() {
        // The service keeps the stored email whenever the body carries a null,
        // so a cleared box would quietly change nothing — say so rather than
        // answer "saved" with the old address still on the row.
        messages.push(format!(
            "email cannot be cleared: the api keeps {} when the box is emptied. replace it instead",
            loaded
                .email
                .clone()
                .unwrap_or_default(),
        ));
    }

    let first_name = wire_text(loaded.first_name.as_deref(), &form.first_name, true);
    let last_name = wire_text(loaded.last_name.as_deref(), &form.last_name, true);

    // A select always carries a value, so empty text means no box was
    // mounted at all: that is no change, and the stored status stays.
    let status = if form.status.trim().is_empty()
        || form.status.trim() == <&'static str>::from(&loaded.status)
    {
        None
    } else {
        match form
            .status
            .trim()
            .parse::<UserStatus>()
        {
            Ok(status) => Some(status),
            Err(err) => {
                messages.push(format!("unknown user status: {err}"));

                None
            },
        }
    };

    let profile = match diff_profile(&loaded.profile, &form.profile) {
        Ok(profile) => profile,
        Err(message) => {
            messages.push(message);

            None
        },
    };

    if !messages.is_empty() {
        return Err(messages);
    }

    Ok(UpdateUserBodyReq {
        user: UpdateUserUser {
            username,
            email,
            first_name,
            last_name,
            status,
            profile,
        },
    })
}

pub fn handle_user_edit_signals(state: AppState, id: Signal<String>) -> HandleUserEditResponse {
    let logout = logout_callback(state);
    let navigate = use_navigate();

    let (user, set_user) = signal(LoadingState::<User>::Pending);
    let (errors, set_errors) = signal(None::<Vec<String>>);
    let (loaded, set_loaded) = signal(None::<LoadedUser>);

    let fetch = Action::new_unsync(move |_: &()| {
        let user_id = user_uuid(id.get_untracked().trim());
        let client = state.client.get_untracked();

        set_errors.set(None);
        set_user.set(LoadingState::Loading);

        async move {
            let user_id = match user_id {
                Ok(user_id) => user_id,
                Err(message) => {
                    set_user.set(LoadingState::Error(message));

                    return;
                },
            };

            match client
                .find_user_by_id(FindUserById { user_id })
                .await
            {
                Ok(res) => {
                    // The table the submit diffs against, taken before the row
                    // moves into the signal the form renders from.
                    set_loaded.set(Some(LoadedUser {
                        username: res.user.username.clone(),
                        email: res.user.email.clone(),
                        first_name: res.user.first_name.clone(),
                        last_name: res.user.last_name.clone(),
                        status: res.user.status.clone(),
                        profile: profile_text(&res.user.profile),
                    }));

                    set_user.set(LoadingState::Loaded(res.user));
                },
                Err(err) => {
                    report_load(err, &logout, set_errors, |message| {
                        set_user.set(LoadingState::Error(message));
                    })
                },
            }
        }
    });

    Effect::new(move |_| {
        fetch.dispatch(());
    });

    let save = Action::new_unsync(move |form: &EditUserForm| {
        let form = form.clone();
        let loaded = loaded.get_untracked();
        let client = state.client.get_untracked();
        let navigate = navigate.clone();

        set_errors.set(None);

        async move {
            // With nothing loaded there is nothing to compare against, and a
            // body built blind would null every field the form did not
            // mention — so refuse instead of guessing.
            let request = match loaded {
                None => Err(vec!["the user has not loaded yet".to_string()]),
                Some(loaded) => build_update_body(&form, &loaded),
            };

            let request = match request {
                Ok(request) => request,
                Err(messages) => {
                    set_errors.set(Some(messages));

                    return;
                },
            };

            let user_id = match user_uuid(&form.id) {
                Ok(user_id) => user_id,
                Err(message) => {
                    set_errors.set(Some(vec![message]));

                    return;
                },
            };

            match client
                .update_user(user_id, request)
                .await
            {
                Ok(res) => navigate(&format!("/users/{}", res.user.id), Default::default()),
                Err(err) => report(err, &logout, set_errors),
            }
        }
    });

    HandleUserEditResponse { user, errors, save }
}

// ---------------------------------------------------------------- detail

#[derive(Clone, Copy)]
pub struct HandleUserDetailResponse {
    pub user: ReadSignal<LoadingState<User>>,
    pub errors: ReadSignal<Option<Vec<String>>>,
}

pub fn handle_user_detail_signals(state: AppState, id: Signal<String>) -> HandleUserDetailResponse {
    let logout = logout_callback(state);

    let (user, set_user) = signal(LoadingState::<User>::Pending);
    let (errors, set_errors) = signal(None::<Vec<String>>);

    let fetch = Action::new_unsync(move |_: &()| {
        let user_id = user_uuid(id.get_untracked().trim());
        let client = state.client.get_untracked();

        set_errors.set(None);
        set_user.set(LoadingState::Loading);

        async move {
            let user_id = match user_id {
                Ok(user_id) => user_id,
                Err(message) => {
                    set_user.set(LoadingState::Error(message));

                    return;
                },
            };

            match client
                .find_user_by_id(FindUserById { user_id })
                .await
            {
                Ok(res) => set_user.set(LoadingState::Loaded(res.user)),
                Err(err) => {
                    report_load(err, &logout, set_errors, |message| {
                        set_user.set(LoadingState::Error(message));
                    })
                },
            }
        }
    });

    Effect::new(move |_| {
        fetch.dispatch(());
    });

    HandleUserDetailResponse { user, errors }
}

// ------------------------------------------------------------- sub-resources
//
// Each section owns its list, its error text, and its own refresh, so a grant
// or a revoke refetches that section alone.

/// The permission text the grant picker hands over: three concrete parts,
/// the same grammar the api's own challenge parser reads.
fn parse_permission(text: &str) -> Result<String, String> {
    let text = text.trim();

    oxidauth_permission::parse::parse(text).map_err(|err| err.to_string())?;

    let parts: Vec<&str> = text.split(':').collect();

    if parts.len() != 3
        || parts
            .iter()
            .any(|part| part.is_empty() || part.contains('*'))
    {
        return Err(
            "permission must be realm:resource:action, e.g. oxidauth:users:read".to_string(),
        );
    }

    Ok(text.to_string())
}

// ---------------------------------------------------------------- roles

#[derive(Clone, Copy)]
pub struct HandleUserRolesResponse {
    pub grants: ReadSignal<LoadingState<Vec<UserRole>>>,
    /// The catalogue the grant picker reads, with its own load state: a
    /// session that may not list roles can still see and revoke what the user
    /// already holds.
    pub catalogue: ReadSignal<LoadingState<Vec<Role>>>,
    pub errors: ReadSignal<Option<Vec<String>>>,
    pub refresh: Action<(), ()>,
    /// Grant, by role id.
    pub grant: Action<String, ()>,
    /// Revoke, by role id.
    pub revoke: Action<String, ()>,
}

pub fn handle_user_roles_signals(
    state: AppState,
    user_id: Signal<String>,
) -> HandleUserRolesResponse {
    let logout = logout_callback(state);

    let (grants, set_grants) = signal(LoadingState::<Vec<UserRole>>::Pending);
    let (catalogue, set_catalogue) = signal(LoadingState::<Vec<Role>>::Pending);
    let (errors, set_errors) = signal(None::<Vec<String>>);

    let refresh = Action::new_unsync(move |_: &()| {
        set_errors.set(None);

        let user_id = resolve_user_id(user_id, set_errors);
        let client = state.client.get_untracked();

        set_grants.set(LoadingState::Loading);

        async move {
            let Some(user_id) = user_id else {
                return;
            };

            match client
                .list_user_roles_by_user_id(user_id)
                .await
            {
                Ok(res) => set_grants.set(LoadingState::Loaded(res.user_role_grants)),
                Err(err) => {
                    report_load(err, &logout, set_errors, |message| {
                        set_grants.set(LoadingState::Error(message));
                    })
                },
            }
        }
    });

    Effect::new(move |_| {
        refresh.dispatch(());
    });

    let load_catalogue = Action::new_unsync(move |_: &()| {
        let client = state.client.get_untracked();

        set_catalogue.set(LoadingState::Loading);

        async move {
            match client
                .list_all_roles(ListAllRoles)
                .await
            {
                Ok(res) => set_catalogue.set(LoadingState::Loaded(res.roles)),
                // The picker says so in its own list rather than stealing the
                // section's error row, which is for the grants themselves.
                Err(err) => {
                    if let Some(message) = handle_feature_error(err.as_ref(), &logout) {
                        set_catalogue.set(LoadingState::Error(message));
                    }
                },
            }
        }
    });

    Effect::new(move |_| {
        load_catalogue.dispatch(());
    });

    let grant = Action::new_unsync(move |role_id: &String| {
        set_errors.set(None);

        let role_id = Uuid::parse_str(role_id.trim()).ok();
        let user_id = resolve_user_id(user_id, set_errors);
        let client = state.client.get_untracked();

        async move {
            let (Some(user_id), Some(role_id)) = (user_id, role_id) else {
                set_errors.set(Some(vec!["pick a role to grant".to_string()]));

                return;
            };

            match client
                .create_user_role(CreateUserRole { user_id, role_id })
                .await
            {
                Ok(_) => {
                    refresh.dispatch(());
                },
                Err(err) => report(err, &logout, set_errors),
            }
        }
    });

    let revoke = Action::new_unsync(move |role_id: &String| {
        set_errors.set(None);

        let role_id = Uuid::parse_str(role_id.trim()).ok();
        let user_id = resolve_user_id(user_id, set_errors);
        let client = state.client.get_untracked();

        async move {
            let (Some(user_id), Some(role_id)) = (user_id, role_id) else {
                set_errors.set(Some(vec!["invalid role id".to_string()]));

                return;
            };

            match client
                .delete_user_role(user_id, role_id)
                .await
            {
                Ok(_) => {
                    refresh.dispatch(());
                },
                Err(err) => report(err, &logout, set_errors),
            }
        }
    });

    HandleUserRolesResponse {
        grants,
        catalogue,
        errors,
        refresh,
        grant,
        revoke,
    }
}

// ----------------------------------------------------------- permissions

#[derive(Clone, Copy)]
pub struct HandleUserPermissionsResponse {
    pub grants: ReadSignal<LoadingState<Vec<UserPermission>>>,
    /// The catalogue the grant picker reads, with its own load state: a
    /// session that may not list permissions can still see and revoke what
    /// the user already holds.
    pub catalogue: ReadSignal<LoadingState<Vec<Permission>>>,
    pub errors: ReadSignal<Option<Vec<String>>>,
    pub refresh: Action<(), ()>,
    /// Grant, by the `realm:resource:action` triple the picker carries.
    pub grant: Action<String, ()>,
    /// Revoke, by the same text read back off the grant.
    pub revoke: Action<String, ()>,
}

pub fn handle_user_permissions_signals(
    state: AppState,
    user_id: Signal<String>,
) -> HandleUserPermissionsResponse {
    let logout = logout_callback(state);

    let (grants, set_grants) = signal(LoadingState::<Vec<UserPermission>>::Pending);
    let (catalogue, set_catalogue) = signal(LoadingState::<Vec<Permission>>::Pending);
    let (errors, set_errors) = signal(None::<Vec<String>>);

    let refresh = Action::new_unsync(move |_: &()| {
        set_errors.set(None);

        let user_id = resolve_user_id(user_id, set_errors);
        let client = state.client.get_untracked();

        set_grants.set(LoadingState::Loading);

        async move {
            let Some(user_id) = user_id else {
                return;
            };

            match client
                .list_user_permission_grants_by_user_id(ListUserPermissionGrantsByUserId {
                    user_id,
                })
                .await
            {
                Ok(res) => set_grants.set(LoadingState::Loaded(res.user_permission_grants)),
                Err(err) => {
                    report_load(err, &logout, set_errors, |message| {
                        set_grants.set(LoadingState::Error(message));
                    })
                },
            }
        }
    });

    Effect::new(move |_| {
        refresh.dispatch(());
    });

    let load_catalogue = Action::new_unsync(move |_: &()| {
        let client = state.client.get_untracked();

        set_catalogue.set(LoadingState::Loading);

        async move {
            match client
                .list_all_permissions(ListAllPermissions)
                .await
            {
                Ok(res) => set_catalogue.set(LoadingState::Loaded(res.permissions)),
                // The picker says so in its own list rather than stealing the
                // section's error row, which is for the grants themselves.
                Err(err) => {
                    if let Some(message) = handle_feature_error(err.as_ref(), &logout) {
                        set_catalogue.set(LoadingState::Error(message));
                    }
                },
            }
        }
    });

    Effect::new(move |_| {
        load_catalogue.dispatch(());
    });

    let grant = Action::new_unsync(move |permission: &String| {
        set_errors.set(None);

        let permission = parse_permission(permission);
        let user_id = resolve_user_id(user_id, set_errors);
        let client = state.client.get_untracked();

        async move {
            let Some(user_id) = user_id else {
                set_errors.set(Some(vec!["pick a permission to grant".to_string()]));

                return;
            };

            let permission = match permission {
                Ok(permission) => permission,
                Err(message) => {
                    set_errors.set(Some(vec![message]));

                    return;
                },
            };

            let params = CreateUserPermission {
                user_id,
                permission,
            };

            match client
                .create_user_permission_grant(params)
                .await
            {
                Ok(_) => {
                    refresh.dispatch(());
                },
                Err(err) => report(err, &logout, set_errors),
            }
        }
    });

    let revoke = Action::new_unsync(move |permission: &String| {
        set_errors.set(None);

        let permission = permission.clone();
        let user_id = resolve_user_id(user_id, set_errors);
        let client = state.client.get_untracked();

        async move {
            let Some(user_id) = user_id else {
                return;
            };

            let params = DeleteUserPermission {
                user_id,
                permission,
            };

            match client
                .delete_user_permission_grant(params)
                .await
            {
                Ok(_) => {
                    refresh.dispatch(());
                },
                Err(err) => report(err, &logout, set_errors),
            }
        }
    });

    HandleUserPermissionsResponse {
        grants,
        catalogue,
        errors,
        refresh,
        grant,
        revoke,
    }
}

// ----------------------------------------------------------- authorities

#[derive(Clone, Copy)]
pub struct HandleUserAuthoritiesResponse {
    pub grants: ReadSignal<LoadingState<Vec<UserAuthorityWithAuthority>>>,
    /// The authorities the grant picker offers, picked by client key.
    pub catalogue: ReadSignal<LoadingState<Vec<Authority>>>,
    pub errors: ReadSignal<Option<Vec<String>>>,
    pub refresh: Action<(), ()>,
    /// Grant, by (client key, params json): the api resolves the authority
    /// from its client key and hands the params to that strategy's registrar.
    pub grant: Action<(String, String), ()>,
    /// Revoke, by authority id.
    pub revoke: Action<String, ()>,
}

pub fn handle_user_authorities_signals(
    state: AppState,
    user_id: Signal<String>,
) -> HandleUserAuthoritiesResponse {
    let logout = logout_callback(state);

    let (grants, set_grants) = signal(LoadingState::<Vec<UserAuthorityWithAuthority>>::Pending);
    let (catalogue, set_catalogue) = signal(LoadingState::<Vec<Authority>>::Pending);
    let (errors, set_errors) = signal(None::<Vec<String>>);

    let refresh = Action::new_unsync(move |_: &()| {
        set_errors.set(None);

        let user_id = resolve_user_id(user_id, set_errors);
        let client = state.client.get_untracked();

        set_grants.set(LoadingState::Loading);

        async move {
            let Some(user_id) = user_id else {
                return;
            };

            match client
                .list_user_authorities_by_user_id(ListUserAuthoritiesByUserId { user_id })
                .await
            {
                Ok(res) => set_grants.set(LoadingState::Loaded(res.user_authorities)),
                Err(err) => {
                    report_load(err, &logout, set_errors, |message| {
                        set_grants.set(LoadingState::Error(message));
                    })
                },
            }
        }
    });

    Effect::new(move |_| {
        refresh.dispatch(());
    });

    let load_catalogue = Action::new_unsync(move |_: &()| {
        let client = state.client.get_untracked();

        set_catalogue.set(LoadingState::Loading);

        async move {
            match client
                .list_all_authorities(ListAllAuthorities {})
                .await
            {
                Ok(res) => set_catalogue.set(LoadingState::Loaded(res.authorities)),
                Err(err) => {
                    if let Some(message) = handle_feature_error(err.as_ref(), &logout) {
                        set_catalogue.set(LoadingState::Error(message));
                    }
                },
            }
        }
    });

    Effect::new(move |_| {
        load_catalogue.dispatch(());
    });

    let grant = Action::new_unsync(move |(client_key, params): &(String, String)| {
        set_errors.set(None);

        let client_key = Uuid::parse_str(client_key.trim()).ok();
        let params = if params.trim().is_empty() {
            Ok(Value::Object(serde_json::Map::new()))
        } else {
            serde_json::from_str::<Value>(params.trim())
                .map_err(|err| format!("params must be valid json: {err}"))
        };
        let user_id = resolve_user_id(user_id, set_errors);
        let client = state.client.get_untracked();

        async move {
            let Some(user_id) = user_id else {
                return;
            };

            let client_key = match client_key {
                Some(client_key) => client_key,
                None => {
                    set_errors.set(Some(vec!["pick an authority to grant".to_string()]));

                    return;
                },
            };

            let params = match params {
                Ok(params) => params,
                Err(message) => {
                    set_errors.set(Some(vec![message]));

                    return;
                },
            };

            let body = CreateUserAuthorityBodyReq {
                client_key,
                user_authority: UserAuthorityParams {
                    params: JsonValue::new(params),
                },
            };

            match client
                .create_user_authority(user_id, body)
                .await
            {
                Ok(_) => {
                    refresh.dispatch(());
                },
                Err(err) => report(err, &logout, set_errors),
            }
        }
    });

    let revoke = Action::new_unsync(move |authority_id: &String| {
        set_errors.set(None);

        let authority_id = Uuid::parse_str(authority_id.trim()).ok();
        let user_id = resolve_user_id(user_id, set_errors);
        let client = state.client.get_untracked();

        async move {
            let Some(user_id) = user_id else {
                return;
            };

            let authority_id = match authority_id {
                Some(authority_id) => authority_id,
                None => {
                    set_errors.set(Some(vec!["invalid authority id".to_string()]));

                    return;
                },
            };

            let params = DeleteUserAuthority {
                user_id,
                authority_id,
            };

            match client
                .delete_user_authority(params)
                .await
            {
                Ok(_) => {
                    refresh.dispatch(());
                },
                Err(err) => report(err, &logout, set_errors),
            }
        }
    });

    HandleUserAuthoritiesResponse {
        grants,
        catalogue,
        errors,
        refresh,
        grant,
        revoke,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn profile() -> Value {
        json!({ "team": "core" })
    }

    fn loaded() -> LoadedUser {
        LoadedUser {
            username: "octocat".to_string(),
            email: Some("octocat@example.com".to_string()),
            first_name: Some("Ada".to_string()),
            last_name: None,
            status: UserStatus::Enabled,
            profile: profile_text(&profile()),
        }
    }

    /// The form as it opens: every box seeded from the row, nothing typed.
    fn untouched(loaded: &LoadedUser) -> EditUserForm {
        EditUserForm {
            id: "11111111-1111-4111-8111-111111111111".to_string(),
            username: loaded.username.clone(),
            email: loaded
                .email
                .clone()
                .unwrap_or_default(),
            first_name: loaded
                .first_name
                .clone()
                .unwrap_or_default(),
            last_name: loaded
                .last_name
                .clone()
                .unwrap_or_default(),
            status: <&'static str>::from(&loaded.status).to_string(),
            profile: loaded.profile.clone(),
        }
    }

    fn user(
        username: &str,
        email: Option<&str>,
        first_name: Option<&str>,
        status: UserStatus,
    ) -> User {
        User {
            id: Uuid::new_v4(),
            kind: UserKind::Human,
            status,
            username: username.to_string(),
            email: email.map(str::to_string),
            first_name: first_name.map(str::to_string),
            last_name: None,
            profile: profile(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn an_untouched_form_writes_nothing_the_row_does_not_already_hold() {
        let loaded = loaded();
        let body = build_update_body(&untouched(&loaded), &loaded)
            .expect("an untouched form is valid")
            .user;

        // The service restores these four from the stored row, so carrying no
        // change for them is how nothing changes.
        assert_eq!(body.username, None);
        assert_eq!(body.email, None);
        assert!(body.status.is_none());
        assert_eq!(body.profile, None);

        // The two names it does not restore (OXA-000015): an omitted name is
        // an erased one, so an untouched name re-travels at its stored text.
        assert_eq!(body.first_name, Some("Ada".to_string()));
        assert_eq!(body.last_name, None, "stored null, so a null writes null");
    }

    #[test]
    fn only_the_field_that_moved_is_written() {
        let loaded = loaded();
        let mut form = untouched(&loaded);
        form.email = "ada@example.com".to_string();

        let body = build_update_body(&form, &loaded)
            .expect("valid")
            .user;

        assert_eq!(body.email, Some("ada@example.com".to_string()));
        assert_eq!(body.username, None, "username never moved");
        assert_eq!(body.first_name, Some("Ada".to_string()));
        assert_eq!(body.profile, None);
        assert!(body.status.is_none());
    }

    #[test]
    fn a_cleared_name_is_a_clear_and_a_cleared_email_is_refused() {
        let loaded = loaded();

        let mut added = untouched(&loaded);
        added.last_name = "Lovelace".to_string();
        let body = build_update_body(&added, &loaded)
            .expect("valid")
            .user;
        assert_eq!(body.last_name, Some("Lovelace".to_string()));

        let mut erased = untouched(&loaded);
        erased.first_name = String::new();
        let body = build_update_body(&erased, &loaded)
            .expect("valid")
            .user;
        assert_eq!(body.first_name, None, "the null is the clear");

        let mut cleared_email = untouched(&loaded);
        cleared_email.email = "  ".to_string();
        let messages =
            build_update_body(&cleared_email, &loaded).expect_err("a clear cannot be sent");

        assert!(
            messages
                .iter()
                .any(|message| message.contains("email cannot be cleared")),
            "{messages:?}"
        );
    }

    #[test]
    fn status_and_profile_travel_only_when_they_change() {
        let loaded = loaded();

        let mut changed = untouched(&loaded);
        changed.status = "disabled".to_string();
        changed.profile = json!({ "team": "ops" }).to_string();
        let body = build_update_body(&changed, &loaded)
            .expect("valid")
            .user;

        assert!(matches!(body.status, Some(UserStatus::Disabled)));
        assert_eq!(body.profile, Some(json!({ "team": "ops" })));

        let mut emptied = untouched(&loaded);
        emptied.profile = String::new();
        let body = build_update_body(&emptied, &loaded)
            .expect("valid")
            .user;
        assert_eq!(
            body.profile,
            Some(json!({})),
            "the column is not-null, so an emptied box means an empty document"
        );

        let mut broken = untouched(&loaded);
        broken.profile = "{ team: core }".to_string();
        let messages = build_update_body(&broken, &loaded).expect_err("invalid json is refused");
        assert!(
            messages
                .iter()
                .any(|message| message.contains("profile must be valid json")),
            "{messages:?}"
        );

        let mut not_a_object = untouched(&loaded);
        not_a_object.profile = "[1, 2]".to_string();
        let messages = build_update_body(&not_a_object, &loaded).expect_err("an array is refused");
        assert!(
            messages
                .iter()
                .any(|message| message.contains("json object")),
            "{messages:?}"
        );

        let mut cleared_username = untouched(&loaded);
        cleared_username.username = String::new();
        let messages =
            build_update_body(&cleared_username, &loaded).expect_err("username is required");
        assert!(
            messages
                .iter()
                .any(|message| message.contains("username is required")),
            "{messages:?}"
        );
    }

    #[test]
    fn the_narrowing_reads_username_email_and_names() {
        let ada = user(
            "octocat",
            Some("ada@example.com"),
            Some("Ada"),
            UserStatus::Enabled,
        );
        let backup = user("svc-backup", None, None, UserStatus::Disabled);

        assert!(user_matches(&ada, "ADA@EXAMPLE", ""), "email, case aside");
        assert!(user_matches(&ada, "oct", ""));
        assert!(
            user_matches(&ada, "ada", ""),
            "matched on the name, not the username"
        );
        assert!(!user_matches(&backup, "ada", ""));
        assert!(user_matches(&backup, "", "disabled"));
        assert!(!user_matches(&ada, "", "disabled"));
        assert!(user_matches(&backup, "", ""), "no narrowing is no filter");
    }

    #[test]
    fn a_permission_is_three_concrete_parts() {
        assert_eq!(
            parse_permission(" oxidauth:users:read ").unwrap(),
            "oxidauth:users:read"
        );

        assert!(parse_permission("oxidauth:users").is_err());
        assert!(parse_permission("**:users:read").is_err());
        assert!(parse_permission("oxidauth::read").is_err());
        assert!(parse_permission("").is_err());
    }
}
