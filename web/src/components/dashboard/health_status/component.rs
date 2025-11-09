use dioxus::prelude::*;
use crate::types::metrics::WalletHealth;

/// Skeleton version of health status for loading states
#[component]
pub fn HealthStatusSkeleton() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div {
            class: "health-status",

            div {
                class: "health-card skeleton",
                style: "--status-color: #666",

                div {
                    class: "skeleton-label",
                    "Loading..."
                }
                div {
                    class: "skeleton-value"
                }
            }

            div {
                class: "health-card skeleton",
                style: "--status-color: #666",

                div {
                    class: "skeleton-label",
                    "Loading..."
                }
                div {
                    class: "skeleton-value skeleton-value-sm"
                }
            }

            div {
                class: "health-card skeleton",
                style: "--status-color: #666",

                div {
                    class: "skeleton-label",
                    "Loading..."
                }
                div {
                    class: "skeleton-value skeleton-value-sm"
                }
            }
        }
    }
}

/// Health status display component showing overall wallet system health
#[component]
pub fn HealthStatus(health: WalletHealth) -> Element {
    let overall_status = if health.healthy { "ONLINE" } else { "OFFLINE" };
    let overall_color = if health.healthy { "#00ff9f" } else { "#ff3333" };
    let btc_status = if health.bitcoin_ready { "READY" } else { "NOT READY" };
    let btc_color = if health.bitcoin_ready { "#00ff9f" } else { "#ff3333" };
    let xmr_status = if health.monero_ready { "READY" } else { "NOT READY" };
    let xmr_color = if health.monero_ready { "#00ff9f" } else { "#ff3333" };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div {
            class: "health-status",

            div {
                class: "health-card",
                style: "--status-color: {overall_color}",

                h4 {
                    class: "health-label",
                    "OVERALL STATUS"
                }
                p {
                    class: "health-value",
                    "{overall_status}"
                }
            }

            div {
                class: "health-card",
                style: "--status-color: {btc_color}",

                h4 {
                    class: "health-label",
                    "[ BTC ] STATUS"
                }
                p {
                    class: "health-value health-value-sm",
                    "{btc_status}"
                }
            }

            div {
                class: "health-card",
                style: "--status-color: {xmr_color}",

                h4 {
                    class: "health-label",
                    "[ XMR ] STATUS"
                }
                p {
                    class: "health-value health-value-sm",
                    "{xmr_status}"
                }
            }
        }
    }
}
