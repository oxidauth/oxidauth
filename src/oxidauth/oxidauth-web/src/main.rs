pub mod components;
pub mod features;
pub mod navigation;
pub mod state;
pub mod time;

use components::app::components::App;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

fn main() {
    // The shell layout (`#app` flex row in style.css) needs a real mount
    // element; mounting into <body> left the sidebar and the page as plain
    // block children, so the nav rail never spanned the viewport.
    let document = web_sys::window()
        .expect("no window")
        .document()
        .expect("no document");
    let app = document
        .get_element_by_id("app")
        .expect("index.html must provide the #app mount root")
        .unchecked_into::<web_sys::HtmlElement>();

    leptos::mount::mount_to(app, || view! { <App /> }).forget();
}
