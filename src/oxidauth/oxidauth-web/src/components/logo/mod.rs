use leptos::prelude::*;

#[component]
pub fn Logo(class: &'static str) -> impl IntoView {
    view! { <img src="logo.svg" class=class alt="oxidauth" /> }
}
