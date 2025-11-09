use dioxus::prelude::*;
use crate::api;
use crate::components::DepositModal;
use crate::types::metrics::WalletBalances;

/// Format a number to 6 significant figures
fn format_significant_figures(value: f64, sig_figs: usize) -> String {
    if value == 0.0 {
        return "0.000000".to_string();
    }
    
    // Calculate the order of magnitude
    let magnitude = value.abs().log10().floor();
    let precision = (sig_figs as i32 - 1 - magnitude as i32).max(0) as usize;
    
    format!("{:.prec$}", value, prec = precision)
}

/// Skeleton version of balance display for loading states
#[component]
pub fn BalanceDisplaySkeleton() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div {
            class: "balance-display",

            div {
                class: "balance-card btc-card skeleton",

                div {
                    class: "balance-glow"
                }

                div {
                    class: "skeleton-title",
                    "Loading..."
                }
                div {
                    class: "skeleton-amount",
                    div { class: "skeleton-value" }
                    div { class: "skeleton-currency" }
                }
                div {
                    class: "skeleton-button"
                }
            }

            div {
                class: "balance-card xmr-card skeleton",

                div {
                    class: "balance-glow"
                }

                div {
                    class: "skeleton-title",
                    "Loading..."
                }
                div {
                    class: "skeleton-amount",
                    div { class: "skeleton-value" }
                    div { class: "skeleton-currency" }
                }
                div {
                    class: "skeleton-button"
                }
            }
        }
    }
}

/// Balance display component showing BTC and XMR balances with deposit buttons
#[component]
pub fn BalanceDisplay(balances: WalletBalances) -> Element {
    let mut show_btc_modal = use_signal(|| false);
    let mut show_xmr_modal = use_signal(|| false);
    let btc_address = use_signal(|| String::new());
    let xmr_address = use_signal(|| String::new());

    // Debug logging
    dioxus_logger::tracing::info!("Rendering balances - BTC: {}, XMR: {}", balances.bitcoin, balances.monero);

    // Format balances to 6 significant figures
    let btc_display = format_significant_figures(balances.bitcoin, 6);
    let xmr_display = format_significant_figures(balances.monero, 6);
    
    // Full precision for tooltips
    let btc_full = format!("{:.8}", balances.bitcoin);
    let xmr_full = format!("{:.12}", balances.monero);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div {
            class: "balance-display",

            div {
                class: "balance-card btc-card",

                div {
                    class: "balance-glow"
                }

                h4 {
                    class: "balance-title btc-title",
                    "// BITCOIN //"
                }
                div {
                    class: "balance-amount",
                    p {
                        class: "balance-value btc-value",
                        title: "{btc_full} BTC",
                        "{btc_display}"
                    }
                    span {
                        class: "balance-currency btc-currency",
                        "BTC"
                    }
                }
                button {
                    class: "deposit-button deposit-button-btc",
                    onclick: move |_| {
                        let mut btc_address = btc_address.clone();
                        let mut show_btc_modal = show_btc_modal.clone();
                        spawn(async move {
                            match api::wallets::fetch_bitcoin_address().await {
                                Ok(addr) => {
                                    btc_address.set(addr.address);
                                    show_btc_modal.set(true);
                                }
                                Err(e) => {
                                    dioxus_logger::tracing::error!("Failed to fetch BTC address: {}", e);
                                }
                            }
                        });
                    },
                    "+ DEPOSIT"
                }
            }

            div {
                class: "balance-card xmr-card",

                div {
                    class: "balance-glow"
                }

                h4 {
                    class: "balance-title xmr-title",
                    "// MONERO //"
                }
                div {
                    class: "balance-amount",
                    p {
                        class: "balance-value xmr-value",
                        title: "{xmr_full} XMR",
                        "{xmr_display}"
                    }
                    span {
                        class: "balance-currency xmr-currency",
                        "XMR"
                    }
                }
                button {
                    class: "deposit-button deposit-button-xmr",
                    onclick: move |_| {
                        let mut xmr_address = xmr_address.clone();
                        let mut show_xmr_modal = show_xmr_modal.clone();
                        spawn(async move {
                            match api::wallets::fetch_monero_address().await {
                                Ok(addr) => {
                                    xmr_address.set(addr.address);
                                    show_xmr_modal.set(true);
                                }
                                Err(e) => {
                                    dioxus_logger::tracing::error!("Failed to fetch XMR address: {}", e);
                                }
                            }
                        });
                    },
                    "+ DEPOSIT"
                }
            }
        }

        // Modals
        DepositModal {
            open: show_btc_modal,
            on_close: move |_| show_btc_modal.set(false),
            coin: "BTC".to_string(),
            address: btc_address(),
            color: "#ffa500".to_string()
        }
        DepositModal {
            open: show_xmr_modal,
            on_close: move |_| show_xmr_modal.set(false),
            coin: "XMR".to_string(),
            address: xmr_address(),
            color: "#ff6b35".to_string()
        }
    }
}
