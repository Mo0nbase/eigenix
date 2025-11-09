use dioxus::prelude::*;
use crate::api;
use crate::components::CharmingChart;
use crate::types::metrics::{BitcoinMetrics, MetricValue};

/// Bitcoin metrics section component
#[component]
pub fn BitcoinMetricsSection(interval: Signal<i64>) -> Element {
    let data = use_resource(move || async move {
        api::metrics::fetch_bitcoin_interval(interval()).await
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div {
            class: "metrics-section btc-metrics",

            h3 {
                class: "metrics-title",
                "// BTC NODE //"
            }

            match data() {
                Some(Ok(metrics)) => rsx! {
                    BitcoinCharts { data: metrics }
                },
                Some(Err(e)) => rsx! {
                    p {
                        class: "error",
                        "ERROR: {e}"
                    }
                },
                None => rsx! {
                    p {
                        class: "loading",
                        "// LOADING BTC METRICS..."
                    }
                }
            }
        }
    }
}

/// Bitcoin charts component
#[component]
pub fn BitcoinCharts(data: Vec<BitcoinMetrics>) -> Element {
    let blocks_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.blocks as f64,
        })
        .collect();

    let progress_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.verification_progress * 100.0,
        })
        .collect();

    let balance_data: Vec<MetricValue> = data
        .iter()
        .filter_map(|m| {
            m.wallet_balance.map(|b| MetricValue {
                timestamp: m.timestamp.clone(),
                value: b,
            })
        })
        .collect();

    rsx! {
        CharmingChart {
            id: "btc-blocks".to_string(),
            title: "BLOCK HEIGHT".to_string(),
            data: blocks_data,
            color: "#ffa500".to_string(),
            y_begin_at_zero: false
        }
        CharmingChart {
            id: "btc-progress".to_string(),
            title: "SYNC PROGRESS %".to_string(),
            data: progress_data,
            color: "#ffa500".to_string(),
            y_begin_at_zero: true
        }
        if !balance_data.is_empty() {
            CharmingChart {
                id: "btc-balance".to_string(),
                title: "WALLET BALANCE BTC".to_string(),
                data: balance_data,
                color: "#ffa500".to_string(),
                y_begin_at_zero: false
            }
        }
    }
}
