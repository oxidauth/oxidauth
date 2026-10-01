use leptos::prelude::*;

/// Renders the error list one page's signals captured (`None` renders
/// nothing). Every feature page keeps its errors local — the same
/// convention as rolodex/parkinglot consoles.
#[component]
pub fn ErrorBanner(errors: ReadSignal<Option<Vec<String>>>) -> impl IntoView {
    view! {
        <Show
            when=move || errors.get().is_some()
            fallback=move || {}
        >
            <div class="form-errors">
                {move || {
                    errors
                        .get()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|err| view! { <p class="form-error">{err}</p> })
                        .collect_view()
                }}
            </div>
        </Show>
    }
}
