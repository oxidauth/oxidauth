use leptos::prelude::*;
use leptos_router::components::Outlet;

use crate::components::layout::components::side_bar::SideBar;

/// The signed-in console shell: sidebar plus the routed page. Reached only
/// through `Protected` routes, so it never renders on the login page.
#[component]
pub fn Layout() -> impl IntoView {
    view! {
        <SideBar />

        <main class="main-content">
            <div class="main-inner">
                <Outlet />
            </div>
        </main>
    }
}
