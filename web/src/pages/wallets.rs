use dioxus::prelude::*;

use crate::api;
use crate::components::Navbar;
use crate::types::metrics::{WalletBalances, WalletHealth};

/// Wallets page displaying Bitcoin and Monero wallet information
#[component]
pub fn Wallets() -> Element {
    // Fetch wallet data
    let balances = use_resource(|| async move { api::wallets::fetch_wallet_balances().await });
    let health = use_resource(|| async move { api::wallets::fetch_wallet_health().await });

    rsx! {
        Navbar {}
        div {
            style: "padding: 20px; max-width: 1200px; margin: 0 auto; color: #e0e0e0; background: #1a1a1a; min-height: 100vh;",
            
            h1 { style: "color: #fff; margin-bottom: 30px; text-align: center;", "💼 Wallet Management" }

            // Wallet Health Status
            div {
                style: "margin: 20px 0; padding: 20px; border: 1px solid #555; border-radius: 12px; background: #2a2a2a;",
                h2 { style: "color: #fff; margin: 0 0 15px 0;", "🏥 Health Status" }
                
                match health() {
                    Some(Ok(health_data)) => rsx! {
                        HealthStatus { health: health_data }
                    },
                    Some(Err(e)) => rsx! {
                        p { style: "color: #f44; font-style: italic;", "Error loading health: {e}" }
                    },
                    None => rsx! {
                        p { style: "color: #aaa; font-style: italic;", "Loading health status..." }
                    }
                }
            }

            // Wallet Balances
            div {
                style: "margin: 20px 0; padding: 20px; border: 1px solid #555; border-radius: 12px; background: #2a2a2a;",
                h2 { style: "color: #fff; margin: 0 0 15px 0;", "💰 Balances" }
                
                match balances() {
                    Some(Ok(balance_data)) => rsx! {
                        BalanceDisplay { balances: balance_data }
                    },
                    Some(Err(e)) => rsx! {
                        p { style: "color: #f44; font-style: italic;", "Error loading balances: {e}" }
                    },
                    None => rsx! {
                        p { style: "color: #aaa; font-style: italic;", "Loading balances..." }
                    }
                }
            }
        }
    }
}

/// Component to display wallet health status
#[component]
fn HealthStatus(health: WalletHealth) -> Element {
    let overall_color = if health.healthy { "#4f4" } else { "#f44" };
    let btc_status = if health.bitcoin_ready { "✅ Ready" } else { "❌ Not Ready" };
    let xmr_status = if health.monero_ready { "✅ Ready" } else { "❌ Not Ready" };

    rsx! {
        div {
            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px;",
            
            div {
                style: "padding: 15px; background: #333; border-radius: 8px;",
                h3 { style: "color: {overall_color}; margin: 0 0 10px 0;", "Overall Status" }
                p { style: "color: #ccc; margin: 0;", if health.healthy { "Healthy" } else { "Unhealthy" } }
            }
            
            div {
                style: "padding: 15px; background: #333; border-radius: 8px;",
                h3 { style: "color: #f7931a; margin: 0 0 10px 0;", "🟠 Bitcoin" }
                p { style: "color: #ccc; margin: 0;", "{btc_status}" }
            }
            
            div {
                style: "padding: 15px; background: #333; border-radius: 8px;",
                h3 { style: "color: #ff6600; margin: 0 0 10px 0;", "🟧 Monero" }
                p { style: "color: #ccc; margin: 0;", "{xmr_status}" }
            }
        }
    }
}

/// Component to display wallet balances
#[component]
fn BalanceDisplay(balances: WalletBalances) -> Element {
    rsx! {
        div {
            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px;",
            
            div {
                style: "padding: 20px; background: linear-gradient(135deg, #333 0%, #444 100%); border-radius: 12px; border: 2px solid #f7931a;",
                h3 { style: "color: #f7931a; margin: 0 0 15px 0; font-size: 1.3rem;", "🟠 Bitcoin" }
                p { 
                    style: "color: #fff; margin: 0; font-size: 1.8rem; font-weight: bold;",
                    "{balances.bitcoin:.8} BTC"
                }
            }
            
            div {
                style: "padding: 20px; background: linear-gradient(135deg, #333 0%, #444 100%); border-radius: 12px; border: 2px solid #ff6600;",
                h3 { style: "color: #ff6600; margin: 0 0 15px 0; font-size: 1.3rem;", "🟧 Monero" }
                p { 
                    style: "color: #fff; margin: 0; font-size: 1.8rem; font-weight: bold;",
                    "{balances.monero:.12} XMR"
                }
            }
        }
    }
}

