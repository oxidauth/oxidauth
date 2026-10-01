use http::data::LoadingState;
use leptos::prelude::*;
use oxidauth::prelude::*;
use oxidauth_kernel::{
    authorities::list_all_authorities::ListAllAuthorities,
    permissions::list_all_permissions::ListAllPermissions,
    roles::list_all_roles::ListAllRoles,
    users::list_all_users::ListAllUsers,
};

use crate::{
    features::auth::signals::{handle_feature_error, logout_callback},
    state::AppState,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct DashboardCounts {
    pub users: usize,
    pub authorities: usize,
    pub roles: usize,
    pub permissions: usize,
}

#[derive(Clone, Copy)]
pub struct HandleDashboardResponse {
    pub counts: ReadSignal<LoadingState<DashboardCounts>>,
    pub refresh: Action<(), ()>,
}

/// One fetch for all four list endpoints — invitations excluded on purpose:
/// the api has no invitation listing, and a scan-all would cost more than
/// the dashboard is worth.
pub fn handle_dashboard_signals(state: AppState) -> HandleDashboardResponse {
    let (counts, set_counts) = signal(LoadingState::<DashboardCounts>::Pending);

    let refresh = Action::new_unsync(move |_: &()| {
        let client = state.client.get_untracked();
        let logout = logout_callback(state);

        set_counts.set(LoadingState::Loading);

        async move {
            let fetched = async {
                let users = client
                    .list_all_users(ListAllUsers)
                    .await
                    .map(|res| res.users.len())?;
                let authorities = client
                    .list_all_authorities(ListAllAuthorities {})
                    .await
                    .map(|res| res.authorities.len())?;
                let roles = client
                    .list_all_roles(ListAllRoles)
                    .await
                    .map(|res| res.roles.len())?;
                let permissions = client
                    .list_all_permissions(ListAllPermissions)
                    .await
                    .map(|res| res.permissions.len())?;
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(DashboardCounts {
                    users,
                    authorities,
                    roles,
                    permissions,
                })
            }
            .await;

            match fetched {
                Ok(counts) => set_counts.set(LoadingState::Loaded(counts)),
                Err(err) => {
                    if let Some(message) = handle_feature_error(err.as_ref(), &logout) {
                        set_counts.set(LoadingState::Error(message));
                    }
                },
            }
        }
    });

    Effect::new(move |_| {
        refresh.dispatch(());
    });

    HandleDashboardResponse { counts, refresh }
}
