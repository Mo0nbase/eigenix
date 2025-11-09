use dioxus::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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
        document::Script { src: "https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js" }
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
                    Some(Some(data)) => {
                        let timestamps: Vec<String> = data.iter().map(|m| m.timestamp.clone()).collect();
                        let labels_json = serde_json::to_string(&timestamps).unwrap_or_default();

                        rsx! {
                            div {
                                if show_btc_blocks() {
                                    {
                                        let values: Vec<u64> = data.iter().map(|m| m.blocks).collect();
                                        let values_json = serde_json::to_string(&values).unwrap_or_default();
                                        rsx! {
                                            h3 { style: "margin-top: 15px;", "Blocks" }
                                            canvas { id: "btc-blocks-chart", style: "max-height: 300px;" }
                                            script {
                                                dangerous_inner_html: r#"
                                                    setTimeout(function() {{
                                                        var ctx = document.getElementById('btc-blocks-chart');
                                                        if (ctx && window.Chart) {{
                                                            if (window.btcBlocksChart) window.btcBlocksChart.destroy();
                                                            window.btcBlocksChart = new Chart(ctx, {{
                                                                type: 'line',
                                                                data: {{
                                                                    labels: {labels_json},
                                                                    datasets: [{{label: 'Blocks', data: {values_json}, borderColor: 'rgb(75, 192, 192)', tension: 0.1}}]
                                                                }},
                                                                options: {{
                                                                    responsive: true,
                                                                    maintainAspectRatio: true,
                                                                    scales: {{ y: {{ beginAtZero: false }} }}
                                                                }}
                                                            }});
                                                        }}
                                                    }}, 100);
                                                "#
                                            }
                                        }
                                    }
                                }
                                if show_btc_headers() {
                                    {
                                        let values: Vec<u64> = data.iter().map(|m| m.headers).collect();
                                        let values_json = serde_json::to_string(&values).unwrap_or_default();
                                        rsx! {
                                            h3 { style: "margin-top: 15px;", "Headers" }
                                            canvas { id: "btc-headers-chart", style: "max-height: 300px;" }
                                            script {
                                                dangerous_inner_html: r#"
                                                    setTimeout(function() {{
                                                        var ctx = document.getElementById('btc-headers-chart');
                                                        if (ctx && window.Chart) {{
                                                            if (window.btcHeadersChart) window.btcHeadersChart.destroy();
                                                            window.btcHeadersChart = new Chart(ctx, {{
                                                                type: 'line',
                                                                data: {{
                                                                    labels: {labels_json},
                                                                    datasets: [{{label: 'Headers', data: {values_json}, borderColor: 'rgb(255, 99, 132)', tension: 0.1}}]
                                                                }},
                                                                options: {{
                                                                    responsive: true,
                                                                    maintainAspectRatio: true,
                                                                    scales: {{ y: {{ beginAtZero: false }} }}
                                                                }}
                                                            }});
                                                        }}
                                                    }}, 100);
                                                "#
                                            }
                                        }
                                    }
                                }
                                if show_btc_verification() {
                                    {
                                        let values: Vec<f64> = data.iter().map(|m| m.verification_progress).collect();
                                        let values_json = serde_json::to_string(&values).unwrap_or_default();
                                        rsx! {
                                            h3 { style: "margin-top: 15px;", "Verification Progress" }
                                            canvas { id: "btc-verification-chart", style: "max-height: 300px;" }
                                            script {
                                                dangerous_inner_html: r#"
                                                    setTimeout(function() {{
                                                        var ctx = document.getElementById('btc-verification-chart');
                                                        if (ctx && window.Chart) {{
                                                            if (window.btcVerificationChart) window.btcVerificationChart.destroy();
                                                            window.btcVerificationChart = new Chart(ctx, {{
                                                                type: 'line',
                                                                data: {{
                                                                    labels: {labels_json},
                                                                    datasets: [{{label: 'Verification Progress', data: {values_json}, borderColor: 'rgb(255, 205, 86)', tension: 0.1}}]
                                                                }},
                                                                options: {{
                                                                    responsive: true,
                                                                    maintainAspectRatio: true,
                                                                    scales: {{ y: {{ beginAtZero: false }} }}
                                                                }}
                                                            }});
                                                        }}
                                                    }}, 100);
                                                "#
                                            }
                                        }
                                    }
                                }
                                if show_btc_size() {
                                    {
                                        let values: Vec<u64> = data.iter().map(|m| m.size_on_disk).collect();
                                        let values_json = serde_json::to_string(&values).unwrap_or_default();
                                        rsx! {
                                            h3 { style: "margin-top: 15px;", "Size on Disk" }
                                            canvas { id: "btc-size-chart", style: "max-height: 300px;" }
                                            script {
                                                dangerous_inner_html: r#"
                                                    setTimeout(function() {{
                                                        var ctx = document.getElementById('btc-size-chart');
                                                        if (ctx && window.Chart) {{
                                                            if (window.btcSizeChart) window.btcSizeChart.destroy();
                                                            window.btcSizeChart = new Chart(ctx, {{
                                                                type: 'line',
                                                                data: {{
                                                                    labels: {labels_json},
                                                                    datasets: [{{label: 'Size on Disk', data: {values_json}, borderColor: 'rgb(153, 102, 255)', tension: 0.1}}]
                                                                }},
                                                                options: {{
                                                                    responsive: true,
                                                                    maintainAspectRatio: true,
                                                                    scales: {{ y: {{ beginAtZero: false }} }}
                                                                }}
                                                            }});
                                                        }}
                                                    }}, 100);
                                                "#
                                            }
                                        }
                                    }
                                }
                                if show_btc_balance() {
                                    {
                                        let values: Vec<Option<f64>> = data.iter().map(|m| m.wallet_balance).collect();
                                        let values_clean: Vec<f64> = values.iter().filter_map(|v| *v).collect();
                                        let values_json = serde_json::to_string(&values_clean).unwrap_or_default();
                                        rsx! {
                                            h3 { style: "margin-top: 15px;", "Wallet Balance" }
                                            canvas { id: "btc-balance-chart", style: "max-height: 300px;" }
                                            script {
                                                dangerous_inner_html: r#"
                                                    setTimeout(function() {{
                                                        var ctx = document.getElementById('btc-balance-chart');
                                                        if (ctx && window.Chart) {{
                                                            if (window.btcBalanceChart) window.btcBalanceChart.destroy();
                                                            window.btcBalanceChart = new Chart(ctx, {{
                                                                type: 'line',
                                                                data: {{
                                                                    labels: {labels_json},
                                                                    datasets: [{{label: 'Wallet Balance', data: {values_json}, borderColor: 'rgb(255, 159, 64)', tension: 0.1}}]
                                                                }},
                                                                options: {{
                                                                    responsive: true,
                                                                    maintainAspectRatio: true,
                                                                    scales: {{ y: {{ beginAtZero: true }} }}
                                                                }}
                                                            }});
                                                        }}
                                                    }}, 100);
                                                "#
                                            }
                                        }
                                    }
                                }
                            }
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
                    Some(Some(data)) => {
                        let timestamps: Vec<String> = data.iter().map(|m| m.timestamp.clone()).collect();
                        let labels_json = serde_json::to_string(&timestamps).unwrap_or_default();

                        rsx! {
                            div {
                                if show_xmr_height() {
                                    {
                                        let values: Vec<u64> = data.iter().map(|m| m.height).collect();
                                        let values_json = serde_json::to_string(&values).unwrap_or_default();
                                        rsx! {
                                            h3 { style: "margin-top: 15px;", "Height" }
                                            canvas { id: "xmr-height-chart", style: "max-height: 300px;" }
                                            script {
                                                dangerous_inner_html: r#"
                                                    setTimeout(function() {{
                                                        var ctx = document.getElementById('xmr-height-chart');
                                                        if (ctx && window.Chart) {{
                                                            if (window.xmrHeightChart) window.xmrHeightChart.destroy();
                                                            window.xmrHeightChart = new Chart(ctx, {{
                                                                type: 'line',
                                                                data: {{
                                                                    labels: {labels_json},
                                                                    datasets: [{{label: 'Height', data: {values_json}, borderColor: 'rgb(75, 192, 192)', tension: 0.1}}]
                                                                }},
                                                                options: {{
                                                                    responsive: true,
                                                                    maintainAspectRatio: true,
                                                                    scales: {{ y: {{ beginAtZero: false }} }}
                                                                }}
                                                            }});
                                                        }}
                                                    }}, 100);
                                                "#
                                            }
                                        }
                                    }
                                }
                                if show_xmr_target_height() {
                                    {
                                        let values: Vec<u64> = data.iter().map(|m| m.target_height).collect();
                                        let values_json = serde_json::to_string(&values).unwrap_or_default();
                                        rsx! {
                                            h3 { style: "margin-top: 15px;", "Target Height" }
                                            canvas { id: "xmr-target-height-chart", style: "max-height: 300px;" }
                                            script {
                                                dangerous_inner_html: r#"
                                                    setTimeout(function() {{
                                                        var ctx = document.getElementById('xmr-target-height-chart');
                                                        if (ctx && window.Chart) {{
                                                            if (window.xmrTargetHeightChart) window.xmrTargetHeightChart.destroy();
                                                            window.xmrTargetHeightChart = new Chart(ctx, {{
                                                                type: 'line',
                                                                data: {{
                                                                    labels: {labels_json},
                                                                    datasets: [{{label: 'Target Height', data: {values_json}, borderColor: 'rgb(255, 99, 132)', tension: 0.1}}]
                                                                }},
                                                                options: {{
                                                                    responsive: true,
                                                                    maintainAspectRatio: true,
                                                                    scales: {{ y: {{ beginAtZero: false }} }}
                                                                }}
                                                            }});
                                                        }}
                                                    }}, 100);
                                                "#
                                            }
                                        }
                                    }
                                }
                                if show_xmr_difficulty() {
                                    {
                                        let values: Vec<u64> = data.iter().map(|m| m.difficulty).collect();
                                        let values_json = serde_json::to_string(&values).unwrap_or_default();
                                        rsx! {
                                            h3 { style: "margin-top: 15px;", "Difficulty" }
                                            canvas { id: "xmr-difficulty-chart", style: "max-height: 300px;" }
                                            script {
                                                dangerous_inner_html: r#"
                                                    setTimeout(function() {{
                                                        var ctx = document.getElementById('xmr-difficulty-chart');
                                                        if (ctx && window.Chart) {{
                                                            if (window.xmrDifficultyChart) window.xmrDifficultyChart.destroy();
                                                            window.xmrDifficultyChart = new Chart(ctx, {{
                                                                type: 'line',
                                                                data: {{
                                                                    labels: {labels_json},
                                                                    datasets: [{{label: 'Difficulty', data: {values_json}, borderColor: 'rgb(255, 205, 86)', tension: 0.1}}]
                                                                }},
                                                                options: {{
                                                                    responsive: true,
                                                                    maintainAspectRatio: true,
                                                                    scales: {{ y: {{ beginAtZero: false }} }}
                                                                }}
                                                            }});
                                                        }}
                                                    }}, 100);
                                                "#
                                            }
                                        }
                                    }
                                }
                                if show_xmr_tx_count() {
                                    {
                                        let values: Vec<u64> = data.iter().map(|m| m.tx_count).collect();
                                        let values_json = serde_json::to_string(&values).unwrap_or_default();
                                        rsx! {
                                            h3 { style: "margin-top: 15px;", "Transaction Count" }
                                            canvas { id: "xmr-tx-count-chart", style: "max-height: 300px;" }
                                            script {
                                                dangerous_inner_html: r#"
                                                    setTimeout(function() {{
                                                        var ctx = document.getElementById('xmr-tx-count-chart');
                                                        if (ctx && window.Chart) {{
                                                            if (window.xmrTxCountChart) window.xmrTxCountChart.destroy();
                                                            window.xmrTxCountChart = new Chart(ctx, {{
                                                                type: 'line',
                                                                data: {{
                                                                    labels: {labels_json},
                                                                    datasets: [{{label: 'Transaction Count', data: {values_json}, borderColor: 'rgb(153, 102, 255)', tension: 0.1}}]
                                                                }},
                                                                options: {{
                                                                    responsive: true,
                                                                    maintainAspectRatio: true,
                                                                    scales: {{ y: {{ beginAtZero: false }} }}
                                                                }}
                                                            }});
                                                        }}
                                                    }}, 100);
                                                "#
                                            }
                                        }
                                    }
                                }
                                if show_xmr_balance() {
                                    {
                                        let values: Vec<Option<f64>> = data.iter().map(|m| m.wallet_balance).collect();
                                        let values_clean: Vec<f64> = values.iter().filter_map(|v| *v).collect();
                                        let values_json = serde_json::to_string(&values_clean).unwrap_or_default();
                                        rsx! {
                                            h3 { style: "margin-top: 15px;", "Wallet Balance" }
                                            canvas { id: "xmr-balance-chart", style: "max-height: 300px;" }
                                            script {
                                                dangerous_inner_html: r#"
                                                    setTimeout(function() {{
                                                        var ctx = document.getElementById('xmr-balance-chart');
                                                        if (ctx && window.Chart) {{
                                                            if (window.xmrBalanceChart) window.xmrBalanceChart.destroy();
                                                            window.xmrBalanceChart = new Chart(ctx, {{
                                                                type: 'line',
                                                                data: {{
                                                                    labels: {labels_json},
                                                                    datasets: [{{label: 'Wallet Balance', data: {values_json}, borderColor: 'rgb(255, 159, 64)', tension: 0.1}}]
                                                                }},
                                                                options: {{
                                                                    responsive: true,
                                                                    maintainAspectRatio: true,
                                                                    scales: {{ y: {{ beginAtZero: true }} }}
                                                                }}
                                                            }});
                                                        }}
                                                    }}, 100);
                                                "#
                                            }
                                        }
                                    }
                                }
                            }
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
                    Some(Some(data)) => {
                        let timestamps: Vec<String> = data.iter().map(|m| m.timestamp.clone()).collect();
                        let labels_json = serde_json::to_string(&timestamps).unwrap_or_default();

                        let mut datasets = Vec::new();
                        if show_asb_balance() {
                            let values: Vec<f64> = data.iter().map(|m| m.balance_btc).collect();
                            datasets.push(format!(r#"{{label: 'Balance BTC', data: {:?}, borderColor: 'rgb(255, 206, 86)', tension: 0.1}}"#, values));
                        }
                        if show_asb_pending() {
                            let values: Vec<u64> = data.iter().map(|m| m.pending_swaps).collect();
                            datasets.push(format!(r#"{{label: 'Pending Swaps', data: {:?}, borderColor: 'rgb(75, 192, 192)', tension: 0.1}}"#, values));
                        }
                        if show_asb_completed() {
                            let values: Vec<u64> = data.iter().map(|m| m.completed_swaps).collect();
                            datasets.push(format!(r#"{{label: 'Completed Swaps', data: {:?}, borderColor: 'rgb(54, 162, 235)', tension: 0.1}}"#, values));
                        }
                        if show_asb_failed() {
                            let values: Vec<u64> = data.iter().map(|m| m.failed_swaps).collect();
                            datasets.push(format!(r#"{{label: 'Failed Swaps', data: {:?}, borderColor: 'rgb(255, 99, 132)', tension: 0.1}}"#, values));
                        }

                        let datasets_json = datasets.join(",");
                        let chart_id = "asb-chart";

                        rsx! {
                            canvas { id: "{chart_id}", style: "max-height: 400px;" }
                            script {
                                dangerous_inner_html: r#"
                                    setTimeout(function() {{
                                        var ctx = document.getElementById('{chart_id}');
                                        if (ctx && window.Chart) {{
                                            if (window.asbChart) window.asbChart.destroy();
                                            window.asbChart = new Chart(ctx, {{
                                                type: 'line',
                                                data: {{
                                                    labels: {labels_json},
                                                    datasets: [{datasets_json}]
                                                }},
                                                options: {{
                                                    responsive: true,
                                                    scales: {{
                                                        y: {{ beginAtZero: true }}
                                                    }}
                                                }}
                                            }});
                                        }}
                                    }}, 100);
                                "#
                            }
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
                    Some(Some(data)) => {
                        let timestamps: Vec<String> = data.iter().map(|m| m.timestamp.clone()).collect();
                        let labels_json = serde_json::to_string(&timestamps).unwrap_or_default();

                        let mut datasets = Vec::new();
                        if show_electrs_blocks() {
                            let values: Vec<u64> = data.iter().map(|m| m.indexed_blocks).collect();
                            datasets.push(format!(r#"{{label: 'Indexed Blocks', data: {:?}, borderColor: 'rgb(255, 159, 64)', tension: 0.1}}"#, values));
                        }

                        let datasets_json = datasets.join(",");
                        let chart_id = "electrs-chart";

                        rsx! {
                            canvas { id: "{chart_id}", style: "max-height: 400px;" }
                            script {
                                dangerous_inner_html: r#"
                                    setTimeout(function() {{
                                        var ctx = document.getElementById('{chart_id}');
                                        if (ctx && window.Chart) {{
                                            if (window.electrsChart) window.electrsChart.destroy();
                                            window.electrsChart = new Chart(ctx, {{
                                                type: 'line',
                                                data: {{
                                                    labels: {labels_json},
                                                    datasets: [{datasets_json}]
                                                }},
                                                options: {{
                                                    responsive: true,
                                                    scales: {{
                                                        y: {{ beginAtZero: false }}
                                                    }}
                                                }}
                                            }});
                                        }}
                                    }}, 100);
                                "#
                            }
                        }
                    },
                    Some(None) => rsx! { p { "Failed to load Electrs metrics" } },
                    None => rsx! { p { "Loading..." } },
                }
            }
        }
    }
}
