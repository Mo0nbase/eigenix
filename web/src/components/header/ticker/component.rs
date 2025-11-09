use dioxus::prelude::*;
use crate::api;
use crate::types::metrics::KrakenTickers;

/// Ticker component displaying Kraken exchange rates in the header
#[component]
pub fn Ticker() -> Element {
    let tickers = use_resource(|| async move {
        api::kraken::fetch_kraken_tickers().await
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div {
            class: "ticker-container",

            match tickers() {
                Some(Ok(ticker_data)) => rsx! {
                    TickerDisplay { tickers: ticker_data }
                },
                Some(Err(e)) => rsx! {
                    div {
                        class: "ticker-error",
                        "Exchange rates unavailable"
                    }
                },
                None => rsx! {
                    div {
                        class: "ticker-loading",
                        "Loading rates..."
                    }
                }
            }
        }
    }
}

/// Display component for ticker data
#[component]
pub fn TickerDisplay(tickers: KrakenTickers) -> Element {
    // Determine change classes based on positive/negative
    let btc_change_class = if tickers.btc_usd_change_24h >= 0.0 { "ticker-change-positive" } else { "ticker-change-negative" };
    let xmr_change_class = if tickers.xmr_usd_change_24h >= 0.0 { "ticker-change-positive" } else { "ticker-change-negative" };
    let pair_change_class = if tickers.xmr_btc_change_24h >= 0.0 { "ticker-change-positive" } else { "ticker-change-negative" };

    rsx! {
        div {
            class: "ticker-display",

            div {
                class: "ticker-item",
                span {
                    class: "ticker-label",
                    "BTC/USD:"
                }
                span {
                    class: "ticker-value ticker-btc",
                    "${tickers.btc_usd:.2}"
                }
                span {
                    class: "ticker-change {btc_change_class}",
                    "{tickers.btc_usd_change_24h:+.2}%"
                }
            }

            div {
                class: "ticker-item",
                span {
                    class: "ticker-label",
                    "XMR/USD:"
                }
                span {
                    class: "ticker-value ticker-xmr",
                    "${tickers.xmr_usd:.2}"
                }
                span {
                    class: "ticker-change {xmr_change_class}",
                    "{tickers.xmr_usd_change_24h:+.2}%"
                }
            }

            div {
                class: "ticker-item",
                span {
                    class: "ticker-label",
                    "XMR/BTC:"
                }
                span {
                    class: "ticker-value ticker-pair",
                    "{tickers.xmr_btc:.8}"
                }
                span {
                    class: "ticker-change {pair_change_class}",
                    "{tickers.xmr_btc_change_24h:+.2}%"
                }
            }
        }
    }
}
