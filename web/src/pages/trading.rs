use dioxus::prelude::*;

use crate::api;
use crate::components::Navbar;
use crate::types::metrics::{TradingConfig, TradingStatus};

/// Trading engine control page
#[component]
pub fn Trading() -> Element {
    // Fetch trading data
    let status = use_resource(|| async move { api::trading::fetch_trading_status().await });
    let config = use_resource(|| async move { api::trading::fetch_trading_config().await });

    rsx! {
        Navbar {}
        div {
            style: "padding: 20px; max-width: 1200px; margin: 0 auto; color: #e0e0e0; background: #1a1a1a; min-height: 100vh;",
            
            h1 { style: "color: #fff; margin-bottom: 30px; text-align: center;", "📈 Trading Engine" }

            // Trading Status
            div {
                style: "margin: 20px 0; padding: 20px; border: 1px solid #555; border-radius: 12px; background: #2a2a2a;",
                h2 { style: "color: #fff; margin: 0 0 15px 0;", "🚦 Status" }
                
                match status() {
                    Some(Ok(status_data)) => rsx! {
                        StatusDisplay { status: status_data }
                    },
                    Some(Err(e)) => rsx! {
                        p { style: "color: #f44; font-style: italic;", "Error loading status: {e}" }
                    },
                    None => rsx! {
                        p { style: "color: #aaa; font-style: italic;", "Loading status..." }
                    }
                }
            }

            // Trading Configuration
            div {
                style: "margin: 20px 0; padding: 20px; border: 1px solid #555; border-radius: 12px; background: #2a2a2a;",
                h2 { style: "color: #fff; margin: 0 0 15px 0;", "⚙️ Configuration" }
                
                match config() {
                    Some(Ok(config_data)) => rsx! {
                        ConfigDisplay { config: config_data }
                    },
                    Some(Err(e)) => rsx! {
                        p { style: "color: #f44; font-style: italic;", "Error loading config: {e}" }
                    },
                    None => rsx! {
                        p { style: "color: #aaa; font-style: italic;", "Loading configuration..." }
                    }
                }
            }
        }
    }
}

/// Component to display trading status
#[component]
fn StatusDisplay(status: TradingStatus) -> Element {
    let status_color = if status.enabled { "#4f4" } else { "#f44" };
    let status_text = if status.enabled { "Enabled" } else { "Disabled" };

    rsx! {
        div {
            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 15px;",
            
            div {
                style: "padding: 15px; background: #333; border-radius: 8px;",
                h3 { style: "color: {status_color}; margin: 0 0 10px 0;", "Engine Status" }
                p { style: "color: #ccc; margin: 0; font-size: 1.2rem; font-weight: bold;", "{status_text}" }
            }
            
            if let Some(last_check) = &status.last_check {
                div {
                    style: "padding: 15px; background: #333; border-radius: 8px;",
                    h3 { style: "color: #aaa; margin: 0 0 10px 0;", "Last Check" }
                    p { style: "color: #ccc; margin: 0;", "{last_check}" }
                }
            }
            
            if let Some(last_trade) = &status.last_trade {
                div {
                    style: "padding: 15px; background: #333; border-radius: 8px;",
                    h3 { style: "color: #aaa; margin: 0 0 10px 0;", "Last Trade" }
                    p { style: "color: #ccc; margin: 0;", "{last_trade}" }
                }
            }
        }
    }
}

/// Component to display trading configuration
#[component]
fn ConfigDisplay(config: TradingConfig) -> Element {
    rsx! {
        div {
            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 15px;",
            
            div {
                style: "padding: 15px; background: #333; border-radius: 8px;",
                h4 { style: "color: #8cf; margin: 0 0 10px 0;", "Target BTC %" }
                p { style: "color: #fff; margin: 0; font-size: 1.3rem;", "{config.target_btc_percentage}%" }
            }
            
            div {
                style: "padding: 15px; background: #333; border-radius: 8px;",
                h4 { style: "color: #8cf; margin: 0 0 10px 0;", "Rebalance Threshold" }
                p { style: "color: #fff; margin: 0; font-size: 1.3rem;", "{config.rebalance_threshold_percentage}%" }
            }
            
            div {
                style: "padding: 15px; background: #333; border-radius: 8px;",
                h4 { style: "color: #8cf; margin: 0 0 10px 0;", "Max Trade Size" }
                p { style: "color: #fff; margin: 0; font-size: 1.3rem;", "{config.max_trade_size_btc} BTC" }
            }
            
            div {
                style: "padding: 15px; background: #333; border-radius: 8px;",
                h4 { style: "color: #8cf; margin: 0 0 10px 0;", "Min Trade Size" }
                p { style: "color: #fff; margin: 0; font-size: 1.3rem;", "{config.min_trade_size_btc} BTC" }
            }
            
            div {
                style: "padding: 15px; background: #333; border-radius: 8px;",
                h4 { style: "color: #8cf; margin: 0 0 10px 0;", "Check Interval" }
                p { style: "color: #fff; margin: 0; font-size: 1.3rem;", "{config.check_interval_seconds}s" }
            }
        }
    }
}

