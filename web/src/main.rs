use dioxus::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
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
    dioxus::launch(App);
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
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://dioxuslabs.com/learn/0.7/", "📚 Learn Dioxus" }
                a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
                a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus", "💫 VSCode Extension" }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        div {
            h1 { "Hello World!" }
            p { "This is the Eigenix home page." }
            a { href: "/metrics", "Go to Metrics" }
        }
    }
}

/// Blog page
#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div {
            id: "blog",

            // Content
            h1 { "This is blog #{id}!" }
            p { "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components." }

            // Navigation links
            Link {
                to: Route::Blog { id: id - 1 },
                "Previous"
            }
            span { " <---> " }
            Link {
                to: Route::Blog { id: id + 1 },
                "Next"
            }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home"
            }
            Link {
                to: Route::Blog { id: 1 },
                "Blog"
            }
            Link {
                to: Route::Metrics {},
                "Metrics"
            }
        }

        Outlet::<Route> {}
    }
}

/// Metrics data structures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct BitcoinMetric {
    timestamp: String,
    blocks: u64,
    headers: u64,
    verification_progress: f64,
    size_on_disk: u64,
    wallet_balance: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct MoneroMetric {
    timestamp: String,
    height: u64,
    target_height: u64,
    difficulty: u64,
    tx_count: u64,
    wallet_balance: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct AsbMetric {
    timestamp: String,
    balance_btc: f64,
    pending_swaps: u64,
    completed_swaps: u64,
    failed_swaps: u64,
    up: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ElectrsMetric {
    timestamp: String,
    up: bool,
    indexed_blocks: u64,
}

/// Metrics page
#[component]
fn Metrics() -> Element {
    let mut interval = use_signal(|| 5i64);

    // Bitcoin field toggles
    let mut show_btc_blocks = use_signal(|| true);
    let mut show_btc_headers = use_signal(|| true);
    let mut show_btc_verification = use_signal(|| true);
    let mut show_btc_size = use_signal(|| true);
    let mut show_btc_balance = use_signal(|| true);

    // Monero field toggles
    let mut show_xmr_height = use_signal(|| true);
    let mut show_xmr_target_height = use_signal(|| true);
    let mut show_xmr_difficulty = use_signal(|| true);
    let mut show_xmr_tx_count = use_signal(|| true);
    let mut show_xmr_balance = use_signal(|| true);

    // ASB field toggles
    let mut show_asb_balance = use_signal(|| true);
    let mut show_asb_pending = use_signal(|| true);
    let mut show_asb_completed = use_signal(|| true);
    let mut show_asb_failed = use_signal(|| true);
    let mut show_asb_up = use_signal(|| true);

    // Electrs field toggles
    let mut show_electrs_up = use_signal(|| true);
    let mut show_electrs_blocks = use_signal(|| true);

    let bitcoin_data = use_resource(move || async move {
        let url = format!(
            "http://localhost:1235/metrics/bitcoin/interval?minutes={}",
            interval()
        );
        Request::get(&url)
            .send()
            .await
            .ok()?
            .json::<Vec<BitcoinMetric>>()
            .await
            .ok()
    });

    let monero_data = use_resource(move || async move {
        let url = format!(
            "http://localhost:1235/metrics/monero/interval?minutes={}",
            interval()
        );
        Request::get(&url)
            .send()
            .await
            .ok()?
            .json::<Vec<MoneroMetric>>()
            .await
            .ok()
    });

    let asb_data = use_resource(move || async move {
        let url = format!(
            "http://localhost:1235/metrics/asb/interval?minutes={}",
            interval()
        );
        Request::get(&url)
            .send()
            .await
            .ok()?
            .json::<Vec<AsbMetric>>()
            .await
            .ok()
    });

    let electrs_data = use_resource(move || async move {
        let url = format!(
            "http://localhost:1235/metrics/electrs/interval?minutes={}",
            interval()
        );
        Request::get(&url)
            .send()
            .await
            .ok()?
            .json::<Vec<ElectrsMetric>>()
            .await
            .ok()
    });

    rsx! {
        div {
            style: "padding: 20px; max-width: 1200px; margin: 0 auto;",
            h1 { "System Metrics" }

            // Interval selector
            div {
                style: "margin: 20px 0;",
                label { "Time Interval: " }
                select {
                    value: "{interval}",
                    onchange: move |evt| {
                        if let Ok(val) = evt.value().parse::<i64>() {
                            interval.set(val);
                        }
                    },
                    option { value: "5", "5 minutes" }
                    option { value: "15", "15 minutes" }
                    option { value: "60", "60 minutes" }
                }
            }

            // Bitcoin metrics
            div {
                style: "margin: 20px 0; padding: 15px; border: 1px solid #333; border-radius: 8px;",
                h2 { "Bitcoin Metrics" }

                // Bitcoin field toggles
                div {
                    style: "margin: 10px 0;",
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_btc_blocks(), onchange: move |evt| show_btc_blocks.set(evt.checked()) } " blocks" }
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_btc_headers(), onchange: move |evt| show_btc_headers.set(evt.checked()) } " headers" }
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_btc_verification(), onchange: move |evt| show_btc_verification.set(evt.checked()) } " verification_progress" }
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_btc_size(), onchange: move |evt| show_btc_size.set(evt.checked()) } " size_on_disk" }
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_btc_balance(), onchange: move |evt| show_btc_balance.set(evt.checked()) } " wallet_balance" }
                }

                match bitcoin_data.read().as_ref() {
                    Some(Some(data)) => rsx! {
                        pre {
                            style: "background: #1a1a1a; padding: 10px; border-radius: 4px; overflow-x: auto;",
                            {data.iter().map(|metric| {
                                let mut fields = Vec::new();
                                fields.push(format!("  \"timestamp\": \"{}\"", metric.timestamp));
                                if show_btc_blocks() { fields.push(format!("  \"blocks\": {}", metric.blocks)); }
                                if show_btc_headers() { fields.push(format!("  \"headers\": {}", metric.headers)); }
                                if show_btc_verification() { fields.push(format!("  \"verification_progress\": {}", metric.verification_progress)); }
                                if show_btc_size() { fields.push(format!("  \"size_on_disk\": {}", metric.size_on_disk)); }
                                if show_btc_balance() { fields.push(format!("  \"wallet_balance\": {:?}", metric.wallet_balance)); }
                                format!("{{\n{}\n}}", fields.join(",\n"))
                            }).collect::<Vec<_>>().join(",\n")}
                        }
                    },
                    Some(None) => rsx! { p { "Failed to load Bitcoin metrics" } },
                    None => rsx! { p { "Loading..." } },
                }
            }

            // Monero metrics
            div {
                style: "margin: 20px 0; padding: 15px; border: 1px solid #333; border-radius: 8px;",
                h2 { "Monero Metrics" }

                // Monero field toggles
                div {
                    style: "margin: 10px 0;",
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_xmr_height(), onchange: move |evt| show_xmr_height.set(evt.checked()) } " height" }
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_xmr_target_height(), onchange: move |evt| show_xmr_target_height.set(evt.checked()) } " target_height" }
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_xmr_difficulty(), onchange: move |evt| show_xmr_difficulty.set(evt.checked()) } " difficulty" }
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_xmr_tx_count(), onchange: move |evt| show_xmr_tx_count.set(evt.checked()) } " tx_count" }
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_xmr_balance(), onchange: move |evt| show_xmr_balance.set(evt.checked()) } " wallet_balance" }
                }

                match monero_data.read().as_ref() {
                    Some(Some(data)) => rsx! {
                        pre {
                            style: "background: #1a1a1a; padding: 10px; border-radius: 4px; overflow-x: auto;",
                            {data.iter().map(|metric| {
                                let mut fields = Vec::new();
                                fields.push(format!("  \"timestamp\": \"{}\"", metric.timestamp));
                                if show_xmr_height() { fields.push(format!("  \"height\": {}", metric.height)); }
                                if show_xmr_target_height() { fields.push(format!("  \"target_height\": {}", metric.target_height)); }
                                if show_xmr_difficulty() { fields.push(format!("  \"difficulty\": {}", metric.difficulty)); }
                                if show_xmr_tx_count() { fields.push(format!("  \"tx_count\": {}", metric.tx_count)); }
                                if show_xmr_balance() { fields.push(format!("  \"wallet_balance\": {:?}", metric.wallet_balance)); }
                                format!("{{\n{}\n}}", fields.join(",\n"))
                            }).collect::<Vec<_>>().join(",\n")}
                        }
                    },
                    Some(None) => rsx! { p { "Failed to load Monero metrics" } },
                    None => rsx! { p { "Loading..." } },
                }
            }

            // ASB metrics
            div {
                style: "margin: 20px 0; padding: 15px; border: 1px solid #333; border-radius: 8px;",
                h2 { "ASB Metrics" }

                // ASB field toggles
                div {
                    style: "margin: 10px 0;",
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_asb_balance(), onchange: move |evt| show_asb_balance.set(evt.checked()) } " balance_btc" }
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_asb_pending(), onchange: move |evt| show_asb_pending.set(evt.checked()) } " pending_swaps" }
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_asb_completed(), onchange: move |evt| show_asb_completed.set(evt.checked()) } " completed_swaps" }
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_asb_failed(), onchange: move |evt| show_asb_failed.set(evt.checked()) } " failed_swaps" }
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_asb_up(), onchange: move |evt| show_asb_up.set(evt.checked()) } " up" }
                }

                match asb_data.read().as_ref() {
                    Some(Some(data)) => rsx! {
                        pre {
                            style: "background: #1a1a1a; padding: 10px; border-radius: 4px; overflow-x: auto;",
                            {data.iter().map(|metric| {
                                let mut fields = Vec::new();
                                fields.push(format!("  \"timestamp\": \"{}\"", metric.timestamp));
                                if show_asb_balance() { fields.push(format!("  \"balance_btc\": {}", metric.balance_btc)); }
                                if show_asb_pending() { fields.push(format!("  \"pending_swaps\": {}", metric.pending_swaps)); }
                                if show_asb_completed() { fields.push(format!("  \"completed_swaps\": {}", metric.completed_swaps)); }
                                if show_asb_failed() { fields.push(format!("  \"failed_swaps\": {}", metric.failed_swaps)); }
                                if show_asb_up() { fields.push(format!("  \"up\": {}", metric.up)); }
                                format!("{{\n{}\n}}", fields.join(",\n"))
                            }).collect::<Vec<_>>().join(",\n")}
                        }
                    },
                    Some(None) => rsx! { p { "Failed to load ASB metrics" } },
                    None => rsx! { p { "Loading..." } },
                }
            }

            // Electrs metrics
            div {
                style: "margin: 20px 0; padding: 15px; border: 1px solid #333; border-radius: 8px;",
                h2 { "Electrs Metrics" }

                // Electrs field toggles
                div {
                    style: "margin: 10px 0;",
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_electrs_up(), onchange: move |evt| show_electrs_up.set(evt.checked()) } " up" }
                    label { style: "margin-right: 15px;", input { r#type: "checkbox", checked: show_electrs_blocks(), onchange: move |evt| show_electrs_blocks.set(evt.checked()) } " indexed_blocks" }
                }

                match electrs_data.read().as_ref() {
                    Some(Some(data)) => rsx! {
                        pre {
                            style: "background: #1a1a1a; padding: 10px; border-radius: 4px; overflow-x: auto;",
                            {data.iter().map(|metric| {
                                let mut fields = Vec::new();
                                fields.push(format!("  \"timestamp\": \"{}\"", metric.timestamp));
                                if show_electrs_up() { fields.push(format!("  \"up\": {}", metric.up)); }
                                if show_electrs_blocks() { fields.push(format!("  \"indexed_blocks\": {}", metric.indexed_blocks)); }
                                format!("{{\n{}\n}}", fields.join(",\n"))
                            }).collect::<Vec<_>>().join(",\n")}
                        }
                    },
                    Some(None) => rsx! { p { "Failed to load Electrs metrics" } },
                    None => rsx! { p { "Loading..." } },
                }
            }
        }
    }
}
