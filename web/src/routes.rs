use dioxus::prelude::*;

use crate::pages::{Home, Metrics, Trading, Wallets};

/// Application routes
#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/metrics")]
    Metrics {},
    #[route("/wallets")]
    Wallets {},
    #[route("/trading")]
    Trading {},
}

