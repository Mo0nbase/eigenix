use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};

mod api;
mod components;
mod constants;
mod pages;
mod routes;
mod types;

use routes::Route;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("Starting Eigenix web application");
    launch(App);
}

/// Root application component
#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}
