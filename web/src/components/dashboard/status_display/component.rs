use crate::api;
use crate::types::metrics::{TradingState, TradingStatus};
use dioxus::prelude::*;

/// Skeleton version of status display for loading states
#[component]
pub fn StatusDisplaySkeleton() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div {
            class: "status-display",

            div {
                class: "status-card skeleton",
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
                class: "status-card status-card-secondary skeleton",

                div {
                    class: "skeleton-label",
                    "Loading..."
                }
                div {
                    class: "skeleton-value skeleton-value-sm"
                }
            }

            div {
                class: "status-card skeleton",
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

/// Trading status display component showing engine state and activity
#[component]
pub fn StatusDisplay(status: TradingStatus) -> Element {
    let mut is_toggling = use_signal(|| false);
    let mut toggle_error = use_signal(|| Option::<String>::None);

    let status_text = if status.enabled {
        "ENABLED"
    } else {
        "DISABLED"
    };
    let status_color = if status.enabled { "#00ff9f" } else { "#ff3333" };

    let state_text = match &status.state {
        TradingState::Disabled => "DISABLED".to_string(),
        TradingState::Monitoring => "MONITORING".to_string(),
        TradingState::DepositingBitcoin { amount } => format!("DEPOSITING BTC ({:.8})", amount),
        TradingState::WaitingForBitcoinDeposit { txid } => {
            format!("WAITING BTC DEPOSIT ({})", &txid[..8])
        }
        TradingState::Trading { btc_amount } => format!("TRADING ({:.8} BTC)", btc_amount),
        TradingState::WaitingForTradeExecution { order_id } => {
            format!("WAITING TRADE ({})", order_id)
        }
        TradingState::WithdrawingMonero { amount } => format!("WITHDRAWING XMR ({:.12})", amount),
        TradingState::WaitingForMoneroWithdrawal { refid } => {
            format!("WAITING XMR WITHDRAWAL ({})", refid)
        }
        TradingState::Error { message } => format!("ERROR: {}", message),
    };

    let state_color = match &status.state {
        TradingState::Disabled => "#666",
        TradingState::Monitoring => "#00d4ff",
        TradingState::DepositingBitcoin { .. } => "#ffaa00",
        TradingState::WaitingForBitcoinDeposit { .. } => "#ffaa00",
        TradingState::Trading { .. } => "#ff00ff",
        TradingState::WaitingForTradeExecution { .. } => "#ff00ff",
        TradingState::WithdrawingMonero { .. } => "#00ff9f",
        TradingState::WaitingForMoneroWithdrawal { .. } => "#00ff9f",
        TradingState::Error { .. } => "#ff3333",
    };

    let on_toggle = move |_| {
        spawn(async move {
            is_toggling.set(true);
            toggle_error.set(None);

            let new_enabled = !status.enabled;
            match api::trading::set_trading_enabled(new_enabled).await {
                Ok(_) => {
                    // Trigger a page reload to refresh status
                    web_sys::window().and_then(|w| w.location().reload().ok());
                }
                Err(e) => {
                    toggle_error.set(Some(format!("Failed to toggle: {}", e)));
                }
            }

            is_toggling.set(false);
        });
    };

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

            div {
                class: "status-card",
                style: "--status-color: {state_color}",

                h4 {
                    class: "status-label",
                    "CURRENT STATE"
                }
                p {
                    class: "status-value status-value-sm",
                    style: "font-size: 11px;",
                    "{state_text}"
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

            if let Some(last_rebalance) = &status.last_rebalance {
                div {
                    class: "status-card",
                    style: "--status-color: #00d4ff",

                    h4 {
                        class: "status-label",
                        "LAST REBALANCE"
                    }
                    p {
                        class: "status-value status-value-sm",
                        "{last_rebalance}"
                    }
                }
            }

            // Control button
            div {
                class: "status-card",
                style: "border: 1px solid {status_color};",

                h4 {
                    class: "status-label",
                    "CONTROLS"
                }
                button {
                    class: "toggle-button",
                    style: "
                        width: 100%;
                        padding: 12px;
                        background: linear-gradient(135deg, {status_color}22, {status_color}11);
                        border: 1px solid {status_color};
                        color: {status_color};
                        font-family: 'Courier New', monospace;
                        font-size: 14px;
                        font-weight: bold;
                        text-transform: uppercase;
                        letter-spacing: 2px;
                        cursor: pointer;
                        transition: all 0.3s ease;
                    ",
                    disabled: is_toggling(),
                    onclick: on_toggle,
                    if is_toggling() {
                        "PROCESSING..."
                    } else if status.enabled {
                        "DISABLE ENGINE"
                    } else {
                        "ENABLE ENGINE"
                    }
                }
            }

            if let Some(error) = toggle_error() {
                div {
                    class: "status-card",
                    style: "--status-color: #ff3333",

                    h4 {
                        class: "status-label",
                        "ERROR"
                    }
                    p {
                        class: "status-value status-value-sm",
                        style: "color: #ff3333; font-size: 10px;",
                        "{error}"
                    }
                }
            }

            // Wallet balances
            if let Some(btc) = status.current_btc_balance {
                div {
                    class: "status-card status-card-secondary",

                    h4 {
                        class: "status-label",
                        "WALLET BTC"
                    }
                    p {
                        class: "status-value status-value-sm",
                        "{btc:.8}"
                    }
                }
            }

            if let Some(xmr) = status.current_xmr_balance {
                div {
                    class: "status-card status-card-secondary",

                    h4 {
                        class: "status-label",
                        "WALLET XMR"
                    }
                    p {
                        class: "status-value status-value-sm",
                        "{xmr:.12}"
                    }
                }
            }

            if let Some(btc) = status.kraken_btc_balance {
                div {
                    class: "status-card status-card-secondary",

                    h4 {
                        class: "status-label",
                        "KRAKEN BTC"
                    }
                    p {
                        class: "status-value status-value-sm",
                        "{btc:.8}"
                    }
                }
            }

            if let Some(xmr) = status.kraken_xmr_balance {
                div {
                    class: "status-card status-card-secondary",

                    h4 {
                        class: "status-label",
                        "KRAKEN XMR"
                    }
                    p {
                        class: "status-value status-value-sm",
                        "{xmr:.12}"
                    }
                }
            }
        }
    }
}
