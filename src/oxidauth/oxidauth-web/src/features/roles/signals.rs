pub use http::data::LoadingState;
use leptos::prelude::*;
use oxidauth::prelude::*;
use oxidauth_http::roles::{create_role::CreateRoleReq, update_role::UpdateRoleReq};
use oxidauth_kernel::{
    error::BoxedError,
    permissions::{Permission, list_all_permissions::ListAllPermissions},
    role_permission_grants::{
        create_role_permission_grant::CreateRolePermissionGrant,
        delete_role_permission_grant::DeleteRolePermissionGrant,
        list_role_permission_grants_by_role_id::ListRolePermissionGrantsByRoleId,
    },
    role_role_grants::{
        create_role_role_grant::CreateRoleRoleGrant,
        delete_role_role_grant::DeleteRoleRoleGrant,
        list_role_role_grants_by_parent_id::ListRoleRoleGrantsByParentId,
    },
    roles::{Role, create_role::CreateRole, list_all_roles::ListAllRoles, update_role::UpdateRole},
};
use uuid::Uuid;

use crate::{
    features::auth::signals::{handle_feature_error, logout_callback},
    state::AppState,
};

/// The one gate every role write needs — create, rename, delete, and both
/// grant mutations all check the same permission server-side.
pub const ROLES_MANAGE: &str = "oxidauth:roles:manage";

/// The row a pending delete confirmation refers to.
#[derive(Clone, Debug, PartialEq)]
pub struct PendingDelete {
    pub id: String,
    pub name: String,
}

/// One granted permission, flattened for the row that shows and revokes it.
/// The grant is addressed by its `realm:resource:action` text — that is the
/// form every grant and revoke endpoint takes.
#[derive(Clone, Debug)]
pub struct PermissionGrantRow {
    pub permission: String,
    pub granted_at: chrono::DateTime<chrono::Utc>,
}

/// One child-role grant: the child record plus when the edge was made.
#[derive(Clone, Debug)]
pub struct ChildRoleRow {
    pub role: Role,
    pub granted_at: chrono::DateTime<chrono::Utc>,
}

/// One boxed SDK error through the session sink: a dead session ends the
/// session and contributes no text (the redirect is the message); anything
/// else is parked in the page's errors signal and also returned, so actions
/// that answer with `Result` can propagate the same text.
fn client_error(
    err: BoxedError,
    logout: &Callback<()>,
    set_errors: WriteSignal<Option<Vec<String>>>,
) -> String {
    match handle_feature_error(err.as_ref(), logout) {
        Some(message) => {
            set_errors.set(Some(vec![message.clone()]));
            message
        },
        None => String::new(),
    }
}

/// Signals and actions for the roles list. The list is unpaginated
/// server-side, so the narrowing is a pure client filter over the loaded
/// rows and the only fetch is the whole list.
#[derive(Clone, Copy)]
pub struct HandleRolesListResponse {
    pub roles: ReadSignal<LoadingState<Vec<Role>>>,
    /// The narrowing in force: the empty string is "no narrowing".
    pub search: ReadSignal<String>,
    pub errors: ReadSignal<Option<Vec<String>>>,
    pub pending_delete: ReadSignal<Option<PendingDelete>>,
    pub apply_search: Callback<String>,
    pub request_delete: Callback<(String, String)>,
    pub cancel_delete: Callback<()>,
    pub confirm_delete: Action<String, ()>,
}

pub fn handle_roles_list_signals(state: AppState) -> HandleRolesListResponse {
    let (roles, set_roles) = signal(LoadingState::<Vec<Role>>::Pending);
    let (search, set_search) = signal(String::new());
    let (errors, set_errors) = signal(None::<Vec<String>>);
    let (pending_delete, set_pending_delete) = signal(None::<PendingDelete>);

    let fetch = Action::new_unsync(move |_: &()| {
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_roles.set(LoadingState::Loading);

        async move {
            match client
                .list_all_roles(ListAllRoles)
                .await
            {
                Ok(res) => set_roles.set(LoadingState::Loaded(res.roles)),
                Err(err) => {
                    if let Some(message) = handle_feature_error(err.as_ref(), &logout) {
                        set_roles.set(LoadingState::Error(message));
                    }
                },
            };
        }
    });

    Effect::new(move |_| {
        fetch.dispatch(());
    });

    let apply_search = Callback::new(move |text: String| {
        set_search.set(text);
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
        let logout = logout_callback(state);

        async move {
            match Uuid::parse_str(&id) {
                Ok(role_id) => {
                    match client
                        .delete_role(role_id)
                        .await
                    {
                        Ok(_) => {
                            set_pending_delete.set(None);
                            fetch.dispatch(());
                        },
                        Err(err) => {
                            set_pending_delete.set(None);
                            client_error(err, &logout, set_errors);
                        },
                    }
                },
                Err(err) => {
                    set_pending_delete.set(None);
                    set_errors.set(Some(vec![err.to_string()]));
                },
            };
        }
    });

    HandleRolesListResponse {
        roles,
        search,
        errors,
        pending_delete,
        apply_search,
        request_delete,
        cancel_delete,
        confirm_delete,
    }
}

/// Single role, fetched by a (signal of) id string.
#[derive(Clone, Copy)]
pub struct HandleRoleDetailResponse {
    pub role: ReadSignal<LoadingState<Role>>,
    pub errors: ReadSignal<Option<Vec<String>>>,
}

pub fn handle_role_detail_signals(state: AppState, id: Signal<String>) -> HandleRoleDetailResponse {
    let (role, set_role) = signal(LoadingState::<Role>::Pending);
    let (errors, set_errors) = signal(None::<Vec<String>>);

    let fetch = Action::new_unsync(move |id: &String| {
        let id = id.clone();
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_errors.set(None);
        set_role.set(LoadingState::Loading);

        async move {
            match Uuid::parse_str(&id) {
                Ok(role_id) => {
                    match client
                        .find_role_by_id(role_id)
                        .await
                    {
                        Ok(res) => set_role.set(LoadingState::Loaded(res.role)),
                        Err(err) => {
                            if let Some(message) = handle_feature_error(err.as_ref(), &logout) {
                                set_role.set(LoadingState::Error(message));
                            }
                        },
                    }
                },
                Err(_) => set_role.set(LoadingState::Error(format!("invalid role id: {id}"))),
            };
        }
    });

    Effect::new(move |_| {
        fetch.dispatch(id.get());
    });

    HandleRoleDetailResponse { role, errors }
}

/// The name as the single-field form submits it; `id` empty means create.
#[derive(Clone, Debug, PartialEq)]
pub struct SaveRoleForm {
    pub id: String,
    pub name: String,
}

/// One factory serves both `/roles/new` and `/roles/:id/edit`: the route
/// decides which. On the new route there is no id, so the preload resolves
/// to `Loaded(None)` and the save creates; on the edit route the current
/// role is preloaded and the save updates.
#[derive(Clone, Copy)]
pub struct HandleRoleFormResponse {
    pub role: ReadSignal<LoadingState<Option<Role>>>,
    pub errors: ReadSignal<Option<Vec<String>>>,
    /// Resolves with the role's id; errors also land in `errors`.
    pub save: Action<SaveRoleForm, Result<Uuid, String>>,
}

pub fn handle_role_form_signals(state: AppState, id: Signal<String>) -> HandleRoleFormResponse {
    let (role, set_role) = signal(LoadingState::<Option<Role>>::Pending);
    let (errors, set_errors) = signal(None::<Vec<String>>);

    let fetch = Action::new_unsync(move |id: &String| {
        let id = id.clone();
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_role.set(LoadingState::Loading);

        async move {
            // Create route: no id means nothing to preload, and nothing is
            // fetched. The check lives inside the async block because the
            // action closure must hand back a future on every path.
            if id.is_empty() {
                set_role.set(LoadingState::Loaded(None));
                return;
            }

            match Uuid::parse_str(&id) {
                Ok(role_id) => {
                    match client
                        .find_role_by_id(role_id)
                        .await
                    {
                        Ok(res) => set_role.set(LoadingState::Loaded(Some(res.role))),
                        Err(err) => {
                            if let Some(message) = handle_feature_error(err.as_ref(), &logout) {
                                set_role.set(LoadingState::Error(message));
                            }
                        },
                    }
                },
                Err(_) => set_role.set(LoadingState::Error(format!("invalid role id: {id}"))),
            };
        }
    });

    Effect::new(move |_| {
        fetch.dispatch(id.get());
    });

    let save = Action::new_unsync(move |form: &SaveRoleForm| {
        let form = form.clone();
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_errors.set(None);

        async move {
            let name = form.name.trim().to_string();
            if name.is_empty() {
                let message = "name is required".to_string();
                set_errors.set(Some(vec![message.clone()]));
                return Err(message);
            }

            if form.id.is_empty() {
                return client
                    .create_role(CreateRoleReq {
                        role: CreateRole { name },
                    })
                    .await
                    .map(|res| res.role.id)
                    .map_err(|err| client_error(err, &logout, set_errors));
            }

            let role_id = match Uuid::parse_str(&form.id) {
                Ok(role_id) => role_id,
                Err(err) => {
                    let message = err.to_string();
                    set_errors.set(Some(vec![message.clone()]));
                    return Err(message);
                },
            };

            client
                .update_role(
                    role_id,
                    UpdateRoleReq {
                        role: UpdateRole {
                            // The path carries the id; the server overwrites
                            // the body's from it.
                            role_id: None,
                            name,
                        },
                    },
                )
                .await
                .map(|res| res.role.id)
                .map_err(|err| client_error(err, &logout, set_errors))
        }
    });

    HandleRoleFormResponse { role, errors, save }
}

/// The role page's permissions section: granted permissions plus the full
/// permission catalog the grant select offers. Fetches and refetches
/// itself, keyed on the (signal of the) role id.
#[derive(Clone, Copy)]
pub struct HandleRolePermissionGrantsResponse {
    pub grants: ReadSignal<LoadingState<Vec<PermissionGrantRow>>>,
    pub catalog: ReadSignal<LoadingState<Vec<Permission>>>,
    pub errors: ReadSignal<Option<Vec<String>>>,
    /// Input: the `realm:resource:action` text of the picked permission.
    pub grant: Action<String, ()>,
    /// Input: the same text, from the row being revoked.
    pub revoke: Action<String, ()>,
}

pub fn handle_role_permission_grants_signals(
    state: AppState,
    role_id: Signal<String>,
) -> HandleRolePermissionGrantsResponse {
    let (grants, set_grants) = signal(LoadingState::<Vec<PermissionGrantRow>>::Pending);
    let (catalog, set_catalog) = signal(LoadingState::<Vec<Permission>>::Pending);
    let (errors, set_errors) = signal(None::<Vec<String>>);

    let fetch = Action::new_unsync(move |id: &String| {
        let id = id.clone();
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_grants.set(LoadingState::Loading);

        async move {
            match Uuid::parse_str(&id) {
                Ok(role_id) => {
                    let params = ListRolePermissionGrantsByRoleId { role_id };
                    match client
                        .list_role_permission_grants_by_role_id(params)
                        .await
                    {
                        Ok(res) => {
                            set_grants.set(LoadingState::Loaded(
                                res.permissions
                                    .into_iter()
                                    .map(|row| {
                                        PermissionGrantRow {
                                            permission: row.permission.to_string(),
                                            granted_at: row.grant.created_at,
                                        }
                                    })
                                    .collect(),
                            ))
                        },
                        Err(err) => {
                            if let Some(message) = handle_feature_error(err.as_ref(), &logout) {
                                set_grants.set(LoadingState::Error(message));
                            }
                        },
                    }
                },
                Err(_) => set_grants.set(LoadingState::Error(format!("invalid role id: {id}"))),
            };
        }
    });

    Effect::new(move |_| {
        fetch.dispatch(role_id.get());
    });

    let fetch_catalog = Action::new_unsync(move |_: &()| {
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_catalog.set(LoadingState::Loading);

        async move {
            match client
                .list_all_permissions(ListAllPermissions)
                .await
            {
                Ok(res) => set_catalog.set(LoadingState::Loaded(res.permissions)),
                Err(err) => {
                    if let Some(message) = handle_feature_error(err.as_ref(), &logout) {
                        set_catalog.set(LoadingState::Error(message));
                    }
                },
            };
        }
    });

    Effect::new(move |_| {
        fetch_catalog.dispatch(());
    });

    let grant = Action::new_unsync(move |permission: &String| {
        let permission = permission.clone();
        let id = role_id.get_untracked();
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_errors.set(None);

        async move {
            match Uuid::parse_str(&id) {
                Ok(role_id) => {
                    let params = CreateRolePermissionGrant {
                        role_id,
                        permission,
                    };
                    match client
                        .create_role_permission_grant(params)
                        .await
                    {
                        Ok(_) => {
                            fetch.dispatch(id);
                        },
                        Err(err) => {
                            // The api's own refusal text is shown verbatim.
                            client_error(err, &logout, set_errors);
                        },
                    }
                },
                Err(err) => set_errors.set(Some(vec![err.to_string()])),
            };
        }
    });

    let revoke = Action::new_unsync(move |permission: &String| {
        let permission = permission.clone();
        let id = role_id.get_untracked();
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_errors.set(None);

        async move {
            match Uuid::parse_str(&id) {
                Ok(role_id) => {
                    let params = DeleteRolePermissionGrant {
                        role_id,
                        permission,
                    };
                    match client
                        .delete_role_permission_grant(params)
                        .await
                    {
                        Ok(_) => {
                            fetch.dispatch(id);
                        },
                        Err(err) => {
                            client_error(err, &logout, set_errors);
                        },
                    }
                },
                Err(err) => set_errors.set(Some(vec![err.to_string()])),
            };
        }
    });

    HandleRolePermissionGrantsResponse {
        grants,
        catalog,
        errors,
        grant,
        revoke,
    }
}

/// The role page's child-roles section: granted children plus the full role
/// list the grant select offers. The server rejects grants that would close
/// a cycle — that refusal is surfaced verbatim in `errors`.
#[derive(Clone, Copy)]
pub struct HandleRoleRoleGrantsResponse {
    pub grants: ReadSignal<LoadingState<Vec<ChildRoleRow>>>,
    pub catalog: ReadSignal<LoadingState<Vec<Role>>>,
    pub errors: ReadSignal<Option<Vec<String>>>,
    /// Input: the child role's id as text.
    pub grant: Action<String, ()>,
    /// Input: the same, from the row being revoked.
    pub revoke: Action<String, ()>,
}

pub fn handle_role_role_grants_signals(
    state: AppState,
    role_id: Signal<String>,
) -> HandleRoleRoleGrantsResponse {
    let (grants, set_grants) = signal(LoadingState::<Vec<ChildRoleRow>>::Pending);
    let (catalog, set_catalog) = signal(LoadingState::<Vec<Role>>::Pending);
    let (errors, set_errors) = signal(None::<Vec<String>>);

    let fetch = Action::new_unsync(move |id: &String| {
        let id = id.clone();
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_grants.set(LoadingState::Loading);

        async move {
            match Uuid::parse_str(&id) {
                Ok(parent_id) => {
                    let params = ListRoleRoleGrantsByParentId { parent_id };
                    match client
                        .list_role_role_grants_by_parent_id(params)
                        .await
                    {
                        Ok(res) => {
                            set_grants.set(LoadingState::Loaded(
                                res.roles
                                    .into_iter()
                                    .map(|detail| {
                                        ChildRoleRow {
                                            role: detail.role,
                                            granted_at: detail.grant.created_at,
                                        }
                                    })
                                    .collect(),
                            ))
                        },
                        Err(err) => {
                            if let Some(message) = handle_feature_error(err.as_ref(), &logout) {
                                set_grants.set(LoadingState::Error(message));
                            }
                        },
                    }
                },
                Err(_) => set_grants.set(LoadingState::Error(format!("invalid role id: {id}"))),
            };
        }
    });

    Effect::new(move |_| {
        fetch.dispatch(role_id.get());
    });

    let fetch_catalog = Action::new_unsync(move |_: &()| {
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_catalog.set(LoadingState::Loading);

        async move {
            match client
                .list_all_roles(ListAllRoles)
                .await
            {
                Ok(res) => set_catalog.set(LoadingState::Loaded(res.roles)),
                Err(err) => {
                    if let Some(message) = handle_feature_error(err.as_ref(), &logout) {
                        set_catalog.set(LoadingState::Error(message));
                    }
                },
            };
        }
    });

    Effect::new(move |_| {
        fetch_catalog.dispatch(());
    });

    let grant = Action::new_unsync(move |child_id: &String| {
        let child_id = child_id.clone();
        let id = role_id.get_untracked();
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_errors.set(None);

        async move {
            match (Uuid::parse_str(&id), Uuid::parse_str(&child_id)) {
                (Ok(parent_id), Ok(child_id)) => {
                    let params = CreateRoleRoleGrant {
                        parent_id,
                        child_id,
                    };
                    match client
                        .create_role_role_grant(params)
                        .await
                    {
                        Ok(_) => {
                            fetch.dispatch(id);
                        },
                        Err(err) => {
                            // Cycle refusals land here, text verbatim.
                            client_error(err, &logout, set_errors);
                        },
                    }
                },
                _ => set_errors.set(Some(vec![format!("invalid role id: {child_id}")])),
            };
        }
    });

    let revoke = Action::new_unsync(move |child_id: &String| {
        let child_id = child_id.clone();
        let id = role_id.get_untracked();
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_errors.set(None);

        async move {
            match (Uuid::parse_str(&id), Uuid::parse_str(&child_id)) {
                (Ok(parent_id), Ok(child_id)) => {
                    let params = DeleteRoleRoleGrant {
                        parent_id,
                        child_id,
                    };
                    match client
                        .delete_role_role_grant(params)
                        .await
                    {
                        Ok(_) => {
                            fetch.dispatch(id);
                        },
                        Err(err) => {
                            client_error(err, &logout, set_errors);
                        },
                    }
                },
                _ => set_errors.set(Some(vec![format!("invalid role id: {child_id}")])),
            };
        }
    });

    HandleRoleRoleGrantsResponse {
        grants,
        catalog,
        errors,
        grant,
        revoke,
    }
}
