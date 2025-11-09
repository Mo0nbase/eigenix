use dioxus::prelude::*;
use crate::types::metrics::TradingStatus;

/// Trading status display component showing engine state and activity
#[component]
pub fn StatusDisplay(status: TradingStatus) -> Element {
    let status_text = if status.enabled { "ENABLED" } else { "DISABLED" };
    let status_color = if status.enabled { "#00d4ff" } else { "#ff3333" };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div {
            class: "status-display",

            div {
                class: "status-card",
                style: "--status-color: {status_color}",

                h4 {
                    class: "status-label",
                    "ENGINE STATE"
                }
                p {
                    class: "status-value",
                    "{status_text}"
                }
            }

            if let Some(last_check) = &status.last_check {
                div {
                    class: "status-card status-card-secondary",

                    h4 {
                        class: "status-label",
                        "LAST CHECK"
                    }
                    p {
                        class: "status-value status-value-sm",
                        "{last_check}"
                    }
                }
            }

            if let Some(last_trade) = &status.last_trade {
                div {
                    class: "status-card",
                    style: "--status-color: #00d4ff",

                    h4 {
                        class: "status-label",
                        "LAST TRADE"
                    }
                    p {
                        class: "status-value status-value-sm",
                        "{last_trade}"
                    }
                }
            }
        }
    }
}
