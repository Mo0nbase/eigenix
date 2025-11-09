use dioxus::prelude::*;

use crate::components::Ticker;
use crate::routes::Route;

/// Navigation bar component with cyberpunk aesthetic
#[component]
pub fn Navbar() -> Element {
    rsx! {
        div { id: "navbar",
            Link {
                to: Route::Dashboard {},
                id: "logo",
                "[ λix ]"
            }
            Ticker {}
        }
    }
}

