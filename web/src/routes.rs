use dioxus::prelude::*;

use crate::pages::Dashboard;

/// Application routes
#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Dashboard {},
}

