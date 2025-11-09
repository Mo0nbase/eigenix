use dioxus::prelude::*;
use crate::types::metrics::TradingConfig;

/// Trading configuration display component showing trading parameters
#[component]
pub fn ConfigDisplay(config: TradingConfig) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div {
            class: "config-display",

            div {
                class: "config-card",
                h5 {
                    class: "config-label",
                    "TARGET BTC %"
                }
                p {
                    class: "config-value",
                    "{config.target_btc_percentage}%"
                }
            }

            div {
                class: "config-card",
                h5 {
                    class: "config-label",
                    "REBALANCE THRESHOLD"
                }
                p {
                    class: "config-value",
                    "{config.rebalance_threshold_percentage}%"
                }
            }

            div {
                class: "config-card",
                h5 {
                    class: "config-label",
                    "MAX TRADE SIZE"
                }
                p {
                    class: "config-value config-value-sm",
                    "{config.max_trade_size_btc} BTC"
                }
            }

            div {
                class: "config-card",
                h5 {
                    class: "config-label",
                    "MIN TRADE SIZE"
                }
                p {
                    class: "config-value config-value-sm",
                    "{config.min_trade_size_btc} BTC"
                }
            }

            div {
                class: "config-card",
                h5 {
                    class: "config-label",
                    "CHECK INTERVAL"
                }
                p {
                    class: "config-value config-value-sm",
                    "{config.check_interval_seconds}s"
                }
            }
        }
    }
}
