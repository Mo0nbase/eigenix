use dioxus::prelude::*;

use crate::routes::Route;

/// Navigation bar component with links to main sections
#[component]
pub fn Navbar() -> Element {
    rsx! {
        div { id: "navbar",
            Link {
                to: Route::Home {},
                id: "logo",
                "🌀 Eigenix"
            }
            div { id: "nav-links",
                Link {
                    to: Route::Home {},
                    "Home"
                }
                Link {
                    to: Route::Metrics {},
                    "Metrics"
                }
                Link {
                    to: Route::Wallets {},
                    "Wallets"
                }
                Link {
                    to: Route::Trading {},
                    "Trading"
                }
            }
        }
    }
}

