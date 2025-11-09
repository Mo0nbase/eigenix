use dioxus::prelude::*;

use crate::components::{Hero, Navbar};

/// Home page component
#[component]
pub fn Home() -> Element {
    rsx! {
        Navbar {}
        Hero {}
    }
}

