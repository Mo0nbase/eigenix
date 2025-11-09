use dioxus::prelude::*;
use crate::api;
use crate::components::CharmingChart;
use crate::types::metrics::{AsbMetrics, MetricValue};

/// ASB metrics section component
#[component]
pub fn AsbMetricsSection(interval: Signal<i64>) -> Element {
    let data =
        use_resource(move || async move { api::metrics::fetch_asb_interval(interval()).await });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div {
            class: "metrics-section asb-metrics",

            h3 {
                class: "metrics-title",
                "// ASB STATUS //"
            }

            match data() {
                Some(Ok(metrics)) => rsx! {
                    AsbCharts { data: metrics }
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
                        "// LOADING ASB METRICS..."
                    }
                }
            }
        }
    }
}

/// ASB charts component
#[component]
pub fn AsbCharts(data: Vec<AsbMetrics>) -> Element {
    let balance_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.balance_btc,
        })
        .collect();

    let pending_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.pending_swaps as f64,
        })
        .collect();

    let completed_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.completed_swaps as f64,
        })
        .collect();

    let failed_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.failed_swaps as f64,
        })
        .collect();

    rsx! {
        CharmingChart {
            id: "asb-balance".to_string(),
            title: "BTC BALANCE".to_string(),
            data: balance_data,
            color: "#00d4ff".to_string(),
            y_begin_at_zero: false
        }
        CharmingChart {
            id: "asb-pending".to_string(),
            title: "PENDING SWAPS".to_string(),
            data: pending_data,
            color: "#ffff00".to_string(),
            y_begin_at_zero: true
        }
        CharmingChart {
            id: "asb-completed".to_string(),
            title: "COMPLETED SWAPS".to_string(),
            data: completed_data,
            color: "#00ff9f".to_string(),
            y_begin_at_zero: true
        }
        CharmingChart {
            id: "asb-failed".to_string(),
            title: "FAILED SWAPS".to_string(),
            data: failed_data,
            color: "#ff3333".to_string(),
            y_begin_at_zero: true
        }
    }
}
