use dioxus::prelude::*;
use crate::api;
use crate::components::CharmingChart;
use crate::types::metrics::{MoneroMetrics, MetricValue};

/// Monero metrics section component
#[component]
pub fn MoneroMetricsSection(interval: Signal<i64>) -> Element {
    let data = use_resource(move || async move {
        api::metrics::fetch_monero_interval(interval()).await
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div {
            class: "metrics-section xmr-metrics",

            h3 {
                class: "metrics-title",
                "// XMR NODE //"
            }

            match data() {
                Some(Ok(metrics)) => rsx! {
                    MoneroCharts { data: metrics }
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
                        "// LOADING XMR METRICS..."
                    }
                }
            }
        }
    }
}

/// Monero charts component
#[component]
pub fn MoneroCharts(data: Vec<MoneroMetrics>) -> Element {
    let height_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.height as f64,
        })
        .collect();

    let difficulty_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.difficulty as f64,
        })
        .collect();

    let tx_count_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.tx_count as f64,
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
            id: "xmr-height".to_string(),
            title: "BLOCK HEIGHT".to_string(),
            data: height_data,
            color: "#ff6b35".to_string(),
            y_begin_at_zero: false
        }
        CharmingChart {
            id: "xmr-difficulty".to_string(),
            title: "NETWORK DIFFICULTY".to_string(),
            data: difficulty_data,
            color: "#ff6b35".to_string(),
            y_begin_at_zero: false
        }
        CharmingChart {
            id: "xmr-txcount".to_string(),
            title: "TRANSACTION COUNT".to_string(),
            data: tx_count_data,
            color: "#ff6b35".to_string(),
            y_begin_at_zero: false
        }
        if !balance_data.is_empty() {
            CharmingChart {
                id: "xmr-balance".to_string(),
                title: "WALLET BALANCE XMR".to_string(),
                data: balance_data,
                color: "#ff6b35".to_string(),
                y_begin_at_zero: false
            }
        }
    }
}
