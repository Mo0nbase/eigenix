use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

mod asb_metrics;
mod bitcoin_metrics;
mod electrs_metrics;
mod monero_metrics;

use asb_metrics::AsbMetrics;
use bitcoin_metrics::BitcoinMetrics;
use electrs_metrics::ElectrsMetrics;
use monero_metrics::MoneroMetrics;

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
    #[route("/metrics")]
    Metrics {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[component]
pub fn Hero() -> Element {
    rsx! {
        div { id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://dioxuslabs.com/learn/0.5/", "📖 Learn Dioxus" }
                a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                a { href: "https://github.com/dioxus-community/", "📋 Community Libraries" }
                a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus", "💫 VSCode Extension" }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        Navbar {}
        Hero {}
    }
}

#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        Navbar {}
        div { id: "blog",
            h1 { "Blog post {id}" }
            p { "This is blog post number {id}." }
            p {
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
            }
            p {
                "Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt."
            }
        }
    }
}

#[component]
fn Navbar() -> Element {
    rsx! {
        div { id: "navbar",
            Link {
                to: Route::Home {},
                id: "logo", "🌀 Eigenix"
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
            }
        }
    }
}

#[component]
fn Metrics() -> Element {
    let mut interval = use_signal(|| 5i64);

    // Category toggles
    let mut show_bitcoin = use_signal(|| true);
    let mut show_monero = use_signal(|| true);
    let mut show_asb = use_signal(|| true);
    let mut show_electrs = use_signal(|| true);

    rsx! {
        Navbar {}
        div {
            style: "padding: 20px; max-width: 1400px; margin: 0 auto; color: #e0e0e0; background: #1a1a1a; min-height: 100vh;",

            // Include Chart.js
            script { src: "https://cdn.jsdelivr.net/npm/chart.js" }

            h1 { style: "color: #fff; margin-bottom: 30px; text-align: center;", "📊 System Metrics Dashboard" }

            // Controls
            div {
                style: "margin: 20px 0; padding: 20px; border: 1px solid #555; border-radius: 12px; background: #2a2a2a;",

                div {
                    style: "display: flex; align-items: center; gap: 20px; margin-bottom: 20px;",
                    label { style: "color: #fff; font-weight: bold;", "Time Interval: " }
                    select {
                        value: "{interval}",
                        style: "padding: 8px 12px; border-radius: 6px; border: 1px solid #555; background: #333; color: #fff;",
                        onchange: move |evt| {
                            if let Ok(val) = evt.value().parse::<i64>() {
                                interval.set(val);
                            }
                        },
                        option { value: "5", "5 minutes" }
                        option { value: "15", "15 minutes" }
                        option { value: "30", "30 minutes" }
                        option { value: "60", "1 hour" }
                        option { value: "360", "6 hours" }
                        option { value: "1440", "24 hours" }
                    }
                }

                div {
                    h3 { style: "color: #fff; margin: 0 0 10px 0;", "Show Categories" }
                    div {
                        style: "display: flex; gap: 20px; flex-wrap: wrap;",
                        label {
                            style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                            input { r#type: "checkbox", checked: show_bitcoin(), onchange: move |evt| show_bitcoin.set(evt.checked()) }
                            span { "🟠 Bitcoin" }
                        }
                        label {
                            style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                            input { r#type: "checkbox", checked: show_monero(), onchange: move |evt| show_monero.set(evt.checked()) }
                            span { "🟧 Monero" }
                        }
                        label {
                            style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                            input { r#type: "checkbox", checked: show_asb(), onchange: move |evt| show_asb.set(evt.checked()) }
                            span { "🔄 ASB" }
                        }
                        label {
                            style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                            input { r#type: "checkbox", checked: show_electrs(), onchange: move |evt| show_electrs.set(evt.checked()) }
                            span { "⚡ Electrs" }
                        }
                    }
                }
            }

            // Bitcoin Metrics
            if show_bitcoin() {
                BitcoinMetrics { interval: interval }
            }

            // Monero Metrics
            if show_monero() {
                MoneroMetrics { interval: interval }
            }

            // ASB Metrics
            if show_asb() {
                AsbMetrics { interval: interval }
            }

            if show_electrs() {
                ElectrsMetrics { interval: interval }
            }
        }
    }
}
