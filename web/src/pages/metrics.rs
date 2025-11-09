use dioxus::prelude::*;

use crate::api;
use crate::components::{CharmingChart, Navbar};
use crate::types::metrics::MetricValue;

/// Main metrics dashboard page
#[component]
pub fn Metrics() -> Element {
    let mut interval = use_signal(|| 5i64);

    // Category toggles
    let mut show_bitcoin = use_signal(|| true);
    let mut show_monero = use_signal(|| true);
    let mut show_asb = use_signal(|| true);

    rsx! {
        Navbar {}
        div {
            style: "padding: 20px; max-width: 1400px; margin: 0 auto; color: #e0e0e0; background: #1a1a1a; min-height: 100vh;",

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
        }
    }
}

/// Bitcoin metrics section component
#[component]
fn BitcoinMetrics(interval: Signal<i64>) -> Element {
    let mut show_blocks = use_signal(|| true);
    let mut show_headers = use_signal(|| true);
    let mut show_verification = use_signal(|| true);
    let mut show_wallet_balance = use_signal(|| true);

    let bitcoin_data = use_resource(move || async move {
        api::metrics::fetch_bitcoin_interval(interval()).await
    });

    rsx! {
        div {
            style: "margin: 30px 0; padding: 20px; border: 1px solid #f39c12; border-radius: 12px; background: linear-gradient(135deg, #1a1a1a 0%, #2a1f0a 100%);",
            h2 {
                style: "color: #f39c12; margin: 0 0 20px 0; font-size: 1.5rem;",
                "🟠 Bitcoin Node Metrics"
            }

            div {
                style: "margin-bottom: 20px;",
                h3 { style: "color: #fff; margin: 0 0 10px 0;", "Show Charts" }
                div {
                    style: "display: flex; gap: 20px; flex-wrap: wrap;",
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input { r#type: "checkbox", checked: show_blocks(), onchange: move |evt| show_blocks.set(evt.checked()) }
                        span { "Blocks" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input { r#type: "checkbox", checked: show_headers(), onchange: move |evt| show_headers.set(evt.checked()) }
                        span { "Headers" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input { r#type: "checkbox", checked: show_verification(), onchange: move |evt| show_verification.set(evt.checked()) }
                        span { "Verification Progress" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input { r#type: "checkbox", checked: show_wallet_balance(), onchange: move |evt| show_wallet_balance.set(evt.checked()) }
                        span { "Wallet Balance" }
                    }
                }
            }

            match bitcoin_data() {
                Some(Ok(data)) if !data.is_empty() => rsx! {
                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(400px, 1fr)); gap: 20px;",
                        
                        if show_blocks() {
                            CharmingChart {
                                id: "btc-blocks".to_string(),
                                title: "Blocks".to_string(),
                                data: data.iter().map(|m| MetricValue {
                                    timestamp: m.timestamp.clone(),
                                    value: m.blocks as f64,
                                }).collect(),
                                color: "rgb(255, 159, 64)".to_string(),
                                y_begin_at_zero: false
                            }
                        }

                        if show_headers() {
                            CharmingChart {
                                id: "btc-headers".to_string(),
                                title: "Headers".to_string(),
                                data: data.iter().map(|m| MetricValue {
                                    timestamp: m.timestamp.clone(),
                                    value: m.headers as f64,
                                }).collect(),
                                color: "rgb(75, 192, 192)".to_string(),
                                y_begin_at_zero: false
                            }
                        }

                        if show_verification() {
                            CharmingChart {
                                id: "btc-verification".to_string(),
                                title: "Verification Progress".to_string(),
                                data: data.iter().map(|m| MetricValue {
                                    timestamp: m.timestamp.clone(),
                                    value: m.verification_progress,
                                }).collect(),
                                color: "rgb(153, 102, 255)".to_string(),
                                y_begin_at_zero: true
                            }
                        }

                        if show_wallet_balance() {
                            CharmingChart {
                                id: "btc-wallet".to_string(),
                                title: "Wallet Balance (BTC)".to_string(),
                                data: data.iter().filter_map(|m| {
                                    m.wallet_balance.map(|balance| MetricValue {
                                        timestamp: m.timestamp.clone(),
                                        value: balance,
                                    })
                                }).collect(),
                                color: "rgb(255, 206, 86)".to_string(),
                                y_begin_at_zero: true
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    p { style: "color: #f44; font-style: italic;", "Error loading Bitcoin metrics: {e}" }
                },
                _ => rsx! {
                    p { style: "color: #aaa; font-style: italic;", "Loading Bitcoin metrics..." }
                }
            }
        }
    }
}

/// Monero metrics section component
#[component]
fn MoneroMetrics(interval: Signal<i64>) -> Element {
    let mut show_height = use_signal(|| true);
    let mut show_target_height = use_signal(|| true);
    let mut show_difficulty = use_signal(|| true);
    let mut show_wallet_balance = use_signal(|| true);

    let monero_data = use_resource(move || async move {
        api::metrics::fetch_monero_interval(interval()).await
    });

    rsx! {
        div {
            style: "margin: 30px 0; padding: 20px; border: 1px solid #ff6600; border-radius: 12px; background: linear-gradient(135deg, #1a1a1a 0%, #2a1500 100%);",
            h2 {
                style: "color: #ff6600; margin: 0 0 20px 0; font-size: 1.5rem;",
                "🟧 Monero Node Metrics"
            }

            div {
                style: "margin-bottom: 20px;",
                h3 { style: "color: #fff; margin: 0 0 10px 0;", "Show Charts" }
                div {
                    style: "display: flex; gap: 20px; flex-wrap: wrap;",
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input { r#type: "checkbox", checked: show_height(), onchange: move |evt| show_height.set(evt.checked()) }
                        span { "Height" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input { r#type: "checkbox", checked: show_target_height(), onchange: move |evt| show_target_height.set(evt.checked()) }
                        span { "Target Height" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input { r#type: "checkbox", checked: show_difficulty(), onchange: move |evt| show_difficulty.set(evt.checked()) }
                        span { "Difficulty" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input { r#type: "checkbox", checked: show_wallet_balance(), onchange: move |evt| show_wallet_balance.set(evt.checked()) }
                        span { "Wallet Balance" }
                    }
                }
            }

            match monero_data() {
                Some(Ok(data)) if !data.is_empty() => rsx! {
                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(400px, 1fr)); gap: 20px;",
                        
                        if show_height() {
                            CharmingChart {
                                id: "xmr-height".to_string(),
                                title: "Height".to_string(),
                                data: data.iter().map(|m| MetricValue {
                                    timestamp: m.timestamp.clone(),
                                    value: m.height as f64,
                                }).collect(),
                                color: "rgb(255, 99, 71)".to_string(),
                                y_begin_at_zero: false
                            }
                        }

                        if show_target_height() {
                            CharmingChart {
                                id: "xmr-target-height".to_string(),
                                title: "Target Height".to_string(),
                                data: data.iter().map(|m| MetricValue {
                                    timestamp: m.timestamp.clone(),
                                    value: m.target_height as f64,
                                }).collect(),
                                color: "rgb(135, 206, 250)".to_string(),
                                y_begin_at_zero: false
                            }
                        }

                        if show_difficulty() {
                            CharmingChart {
                                id: "xmr-difficulty".to_string(),
                                title: "Difficulty".to_string(),
                                data: data.iter().map(|m| MetricValue {
                                    timestamp: m.timestamp.clone(),
                                    value: m.difficulty as f64,
                                }).collect(),
                                color: "rgb(255, 140, 0)".to_string(),
                                y_begin_at_zero: false
                            }
                        }

                        if show_wallet_balance() {
                            CharmingChart {
                                id: "xmr-wallet".to_string(),
                                title: "Wallet Balance (XMR)".to_string(),
                                data: data.iter().filter_map(|m| {
                                    m.wallet_balance.map(|balance| MetricValue {
                                        timestamp: m.timestamp.clone(),
                                        value: balance,
                                    })
                                }).collect(),
                                color: "rgb(144, 238, 144)".to_string(),
                                y_begin_at_zero: true
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    p { style: "color: #f44; font-style: italic;", "Error loading Monero metrics: {e}" }
                },
                _ => rsx! {
                    p { style: "color: #aaa; font-style: italic;", "Loading Monero metrics..." }
                }
            }
        }
    }
}

/// ASB (Atomic Swap Bot) metrics section component
#[component]
fn AsbMetrics(interval: Signal<i64>) -> Element {
    let mut show_balance = use_signal(|| true);
    let mut show_pending = use_signal(|| true);
    let mut show_completed = use_signal(|| true);
    let mut show_failed = use_signal(|| true);

    let asb_data = use_resource(move || async move {
        api::metrics::fetch_asb_interval(interval()).await
    });

    rsx! {
        div {
            style: "margin: 30px 0; padding: 20px; border: 1px solid #3498db; border-radius: 12px; background: linear-gradient(135deg, #1a1a1a 0%, #0a1a2a 100%);",
            h2 {
                style: "color: #3498db; margin: 0 0 20px 0; font-size: 1.5rem;",
                "🔄 Atomic Swap Bot Metrics"
            }

            div {
                style: "margin-bottom: 20px;",
                h3 { style: "color: #fff; margin: 0 0 10px 0;", "Show Charts" }
                div {
                    style: "display: flex; gap: 20px; flex-wrap: wrap;",
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input { r#type: "checkbox", checked: show_balance(), onchange: move |evt| show_balance.set(evt.checked()) }
                        span { "BTC Balance" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input { r#type: "checkbox", checked: show_pending(), onchange: move |evt| show_pending.set(evt.checked()) }
                        span { "Pending Swaps" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input { r#type: "checkbox", checked: show_completed(), onchange: move |evt| show_completed.set(evt.checked()) }
                        span { "Completed Swaps" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input { r#type: "checkbox", checked: show_failed(), onchange: move |evt| show_failed.set(evt.checked()) }
                        span { "Failed Swaps" }
                    }
                }
            }

            match asb_data() {
                Some(Ok(data)) if !data.is_empty() => rsx! {
                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(400px, 1fr)); gap: 20px;",
                        
                        if show_balance() {
                            CharmingChart {
                                id: "asb-btc".to_string(),
                                title: "BTC Balance".to_string(),
                                data: data.iter().map(|m| MetricValue {
                                    timestamp: m.timestamp.clone(),
                                    value: m.balance_btc,
                                }).collect(),
                                color: "rgb(255, 193, 7)".to_string(),
                                y_begin_at_zero: true
                            }
                        }

                        if show_pending() {
                            CharmingChart {
                                id: "asb-pending".to_string(),
                                title: "Pending Swaps".to_string(),
                                data: data.iter().map(|m| MetricValue {
                                    timestamp: m.timestamp.clone(),
                                    value: m.pending_swaps as f64,
                                }).collect(),
                                color: "rgb(255, 193, 7)".to_string(),
                                y_begin_at_zero: true
                            }
                        }

                        if show_completed() {
                            CharmingChart {
                                id: "asb-completed".to_string(),
                                title: "Completed Swaps".to_string(),
                                data: data.iter().map(|m| MetricValue {
                                    timestamp: m.timestamp.clone(),
                                    value: m.completed_swaps as f64,
                                }).collect(),
                                color: "rgb(76, 175, 80)".to_string(),
                                y_begin_at_zero: true
                            }
                        }

                        if show_failed() {
                            CharmingChart {
                                id: "asb-failed".to_string(),
                                title: "Failed Swaps".to_string(),
                                data: data.iter().map(|m| MetricValue {
                                    timestamp: m.timestamp.clone(),
                                    value: m.failed_swaps as f64,
                                }).collect(),
                                color: "rgb(244, 67, 54)".to_string(),
                                y_begin_at_zero: true
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    p { style: "color: #f44; font-style: italic;", "Error loading ASB metrics: {e}" }
                },
                _ => rsx! {
                    p { style: "color: #aaa; font-style: italic;", "Loading ASB metrics..." }
                }
            }
        }
    }
}

