use crate::types::metrics::TradingConfig;
use dioxus::prelude::*;

/// Skeleton version of config display for loading states
#[component]
pub fn ConfigDisplaySkeleton() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div {
            class: "config-display",

            div {
                class: "config-card skeleton",
                div {
                    class: "skeleton-label",
                    "Loading..."
                }
                div {
                    class: "skeleton-value"
                }
            }

            div {
                class: "config-card skeleton",
                div {
                    class: "skeleton-label",
                    "Loading..."
                }
                div {
                    class: "skeleton-value"
                }
            }

            div {
                class: "config-card skeleton",
                div {
                    class: "skeleton-label",
                    "Loading..."
                }
                div {
                    class: "skeleton-value skeleton-value-sm"
                }
            }

            div {
                class: "config-card skeleton",
                div {
                    class: "skeleton-label",
                    "Loading..."
                }
                div {
                    class: "skeleton-value skeleton-value-sm"
                }
            }

            div {
                class: "config-card skeleton",
                div {
                    class: "skeleton-label",
                    "Loading..."
                }
                div {
                    class: "skeleton-value skeleton-value-sm"
                }
            }

            div {
                class: "config-card skeleton",
                div {
                    class: "skeleton-label",
                    "Loading..."
                }
                div {
                    class: "skeleton-value skeleton-value-sm"
                }
            }

            div {
                class: "config-card skeleton",
                div {
                    class: "skeleton-label",
                    "Loading..."
                }
                div {
                    class: "skeleton-value skeleton-value-sm"
                }
            }

            div {
                class: "config-card skeleton",
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
                    "XMR MIN THRESHOLD"
                }
                p {
                    class: "config-value",
                    "{config.monero_min_threshold:.4} XMR"
                }
            }

            div {
                class: "config-card",
                h5 {
                    class: "config-label",
                    "XMR TARGET BALANCE"
                }
                p {
                    class: "config-value",
                    "{config.monero_target_balance:.4} XMR"
                }
            }

            div {
                class: "config-card",
                h5 {
                    class: "config-label",
                    "BTC RESERVE MIN"
                }
                p {
                    class: "config-value config-value-sm",
                    "{config.bitcoin_reserve_minimum:.8} BTC"
                }
            }

            div {
                class: "config-card",
                h5 {
                    class: "config-label",
                    "MAX BTC PER REBALANCE"
                }
                p {
                    class: "config-value config-value-sm",
                    "{config.max_btc_per_rebalance:.8} BTC"
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
                    "{config.check_interval_secs}s"
                }
            }

            div {
                class: "config-card",
                h5 {
                    class: "config-label",
                    "ORDER TIMEOUT"
                }
                p {
                    class: "config-value config-value-sm",
                    "{config.order_timeout_secs}s"
                }
            }

            div {
                class: "config-card",
                h5 {
                    class: "config-label",
                    "SLIPPAGE TOLERANCE"
                }
                p {
                    class: "config-value config-value-sm",
                    "{config.slippage_tolerance_percent}%"
                }
            }

            div {
                class: "config-card",
                h5 {
                    class: "config-label",
                    "ORDER TYPE"
                }
                p {
                    class: "config-value config-value-sm",
                    if config.use_limit_orders {
                        "LIMIT ORDERS"
                    } else {
                        "MARKET ORDERS"
                    }
                }
            }
        }
    }
}
