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
            }

            div {
                class: "ticker-item",
                span {
                    class: "ticker-label",
                    "BTC/XMR:"
                }
                span {
                    class: "ticker-value ticker-pair",
                    "{tickers.btc_xmr:.2}"
                }
            }
        }
    }
}
