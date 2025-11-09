use dioxus::prelude::*;
use crate::api;
use crate::components::DepositModal;
use crate::types::metrics::WalletBalances;

/// Balance display component showing BTC and XMR balances with deposit buttons
#[component]
pub fn BalanceDisplay(balances: WalletBalances) -> Element {
    let mut show_btc_modal = use_signal(|| false);
    let mut show_xmr_modal = use_signal(|| false);
    let btc_address = use_signal(|| String::new());
    let xmr_address = use_signal(|| String::new());

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
                        "{balances.bitcoin:.8}"
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
                        "{balances.monero:.12}"
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
