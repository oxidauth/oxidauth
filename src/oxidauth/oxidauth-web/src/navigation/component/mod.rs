use leptos::prelude::*;
use leptos_router::{components::*, path};

use crate::{
    components::layout::components::layout::Layout,
    features::{
        auth::components::{LoginPage, Protected},
        authorities::components::{AuthoritiesPage, AuthorityFormPage},
        dashboard::components::DashboardPage,
        invitations::components::{InvitationDetailPage, InvitationFormPage, InvitationsPage},
        permissions::components::PermissionsPage,
        roles::components::{RoleDetailPage, RoleFormPage, RolesPage},
        users::components::{UserDetailPage, UserFormPage, UsersPage},
    },
};

/// Routes. `/` and `/login` render bare; every signed-in page renders inside
/// the [`Layout`] shell via `ParentRoute`, so the navigation can never appear
/// on the login page, and `Protected` wraps every shell page so a dead
/// session bounces to the login form.
#[component]
pub fn Navigation() -> impl IntoView {
    view! {
        <Routes fallback=|| "Not Found">
            <Route path=path!("/") view=LoginPage />
            <Route path=path!("/login") view=LoginPage />
            <ParentRoute path=path!("") view=Layout>
                <Route
                    path=path!("/dashboard")
                    view=|| view! { <Protected><DashboardPage /></Protected> }
                />
                <Route
                    path=path!("/users")
                    view=|| view! { <Protected><UsersPage /></Protected> }
                />
                <Route
                    path=path!("/users/new")
                    view=|| view! { <Protected><UserFormPage /></Protected> }
                />
                <Route
                    path=path!("/users/:id")
                    view=|| view! { <Protected><UserDetailPage /></Protected> }
                />
                <Route
                    path=path!("/users/:id/edit")
                    view=|| view! { <Protected><UserFormPage /></Protected> }
                />
                <Route
                    path=path!("/authorities")
                    view=|| view! { <Protected><AuthoritiesPage /></Protected> }
                />
                <Route
                    path=path!("/authorities/new")
                    view=|| view! { <Protected><AuthorityFormPage /></Protected> }
                />
                <Route
                    path=path!("/authorities/:id/edit")
                    view=|| view! { <Protected><AuthorityFormPage /></Protected> }
                />
                <Route
                    path=path!("/roles")
                    view=|| view! { <Protected><RolesPage /></Protected> }
                />
                <Route
                    path=path!("/roles/new")
                    view=|| view! { <Protected><RoleFormPage /></Protected> }
                />
                <Route
                    path=path!("/roles/:id")
                    view=|| view! { <Protected><RoleDetailPage /></Protected> }
                />
                <Route
                    path=path!("/roles/:id/edit")
                    view=|| view! { <Protected><RoleFormPage /></Protected> }
                />
                <Route
                    path=path!("/permissions")
                    view=|| view! { <Protected><PermissionsPage /></Protected> }
                />
                <Route
                    path=path!("/invitations")
                    view=|| view! { <Protected><InvitationsPage /></Protected> }
                />
                <Route
                    path=path!("/invitations/new")
                    view=|| view! { <Protected><InvitationFormPage /></Protected> }
                />
                <Route
                    path=path!("/invitations/:id")
                    view=|| view! { <Protected><InvitationDetailPage /></Protected> }
                />
            </ParentRoute>
        </Routes>
    }
}
