use crate::api;
use crate::types::metrics::{TradingConfig, TradingState, TradingStatus};
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
                    class: "skeleton-value skeleton-value-sm"
                }
            }

            div {
                class: "status-card skeleton",
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

/// Trading status display component showing engine state, activity, and configuration
#[component]
pub fn StatusDisplay(status: TradingStatus, config: TradingConfig) -> Element {
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

    let state_tooltip = match &status.state {
        TradingState::Disabled => "Engine is not running",
        TradingState::Monitoring => "Actively monitoring balances for rebalancing opportunities",
        TradingState::DepositingBitcoin { .. } => "Sending Bitcoin to Kraken exchange",
        TradingState::WaitingForBitcoinDeposit { .. } => {
            "Waiting for Bitcoin deposit confirmation on Kraken"
        }
        TradingState::Trading { .. } => "Executing BTC→XMR trade on Kraken",
        TradingState::WaitingForTradeExecution { .. } => "Waiting for trade order to complete",
        TradingState::WithdrawingMonero { .. } => "Withdrawing Monero from Kraken to local wallet",
        TradingState::WaitingForMoneroWithdrawal { .. } => {
            "Waiting for Monero withdrawal to complete"
        }
        TradingState::Error { .. } => "An error occurred during operation",
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

            // Control button - full width at top
            div {
                class: "status-card status-card-full-width",
                style: "border: 1px solid #666;",

                h4 {
                    class: "status-label",
                    "CONTROLS"
                }
                button {
                    class: if status.enabled { "toggle-button toggle-button-disable" } else { "toggle-button toggle-button-enable" },
                    style: "
                        width: 100%;
                        padding: 12px;
                        background: #0a0a0a;
                        border: 1px solid #666;
                        color: #999;
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

            // Grid container for remaining cards
            div {
                class: "status-grid",

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
                    title: "{state_tooltip}",

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

                // Configuration parameters
                div {
                    class: "status-card",
                    h5 {
                        class: "status-label",
                        "XMR MIN THRESHOLD"
                    }
                    p {
                        class: "status-value status-value-sm",
                        "{config.monero_min_threshold:.4} XMR"
                    }
                }

                div {
                    class: "status-card",
                    h5 {
                        class: "status-label",
                        "XMR TARGET BALANCE"
                    }
                    p {
                        class: "status-value status-value-sm",
                        "{config.monero_target_balance:.4} XMR"
                    }
                }

                div {
                    class: "status-card",
                    h5 {
                        class: "status-label",
                        "BTC RESERVE MIN"
                    }
                    p {
                        class: "status-value status-value-sm",
                        "{config.bitcoin_reserve_minimum:.8} BTC"
                    }
                }

                div {
                    class: "status-card",
                    h5 {
                        class: "status-label",
                        "MAX BTC PER REBALANCE"
                    }
                    p {
                        class: "status-value status-value-sm",
                        "{config.max_btc_per_rebalance:.8} BTC"
                    }
                }

                div {
                    class: "status-card",
                    h5 {
                        class: "status-label",
                        "CHECK INTERVAL"
                    }
                    p {
                        class: "status-value status-value-sm",
                        "{config.check_interval_secs}s"
                    }
                }

                div {
                    class: "status-card",
                    h5 {
                        class: "status-label",
                        "ORDER TIMEOUT"
                    }
                    p {
                        class: "status-value status-value-sm",
                        "{config.order_timeout_secs}s"
                    }
                }

                div {
                    class: "status-card",
                    h5 {
                        class: "status-label",
                        "SLIPPAGE TOLERANCE"
                    }
                    p {
                        class: "status-value status-value-sm",
                        "{config.slippage_tolerance_percent}%"
                    }
                }

                div {
                    class: "status-card",
                    h5 {
                        class: "status-label",
                        "ORDER TYPE"
                    }
                    p {
                        class: "status-value status-value-sm",
                        if config.use_limit_orders {
                            "LIMIT ORDERS"
                        } else {
                            "MARKET ORDERS"
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

                if let Some(error) = toggle_error() {
                    div {
                        class: "status-card",
                        style: "--status-color: #ff3333; grid-column: 1 / -1;",

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
            }
        }
    }
}
