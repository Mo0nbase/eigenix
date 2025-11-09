use dioxus::prelude::*;

use crate::api;
use crate::components::{Navbar, dashboard::*};

/// Unified dashboard combining Metrics, Wallets, and Trading
#[component]
pub fn Dashboard() -> Element {
    let mut interval = use_signal(|| 5i64);

    // Category toggles
    let mut show_bitcoin = use_signal(|| true);
    let mut show_monero = use_signal(|| true);
    let mut show_asb = use_signal(|| true);

    // Fetch wallet data
    let balances = use_resource(|| async move { api::wallets::fetch_wallet_balances().await });
    let health = use_resource(|| async move { api::wallets::fetch_wallet_health().await });

    // Fetch trading data
    let status = use_resource(|| async move { api::trading::fetch_trading_status().await });
    let config = use_resource(|| async move { api::trading::fetch_trading_config().await });

    rsx! {
        Navbar {}
        div {
            style: "padding: 40px; max-width: 1600px; margin: 0 auto; min-height: 100vh;",

            h1 {
                style: "color: #fff; margin-bottom: 40px; text-align: center; font-size: 32px; text-transform: uppercase; letter-spacing: 4px; text-shadow: 0 0 20px rgba(255,255,255,0.8);",
                "[ λix DASHBOARD ]"
            }

            // TOP ROW: WALLETS AND TRADING SIDE BY SIDE
            div {
                style: "display: grid; grid-template-columns: 1fr 1fr; gap: 30px; margin-bottom: 60px;",

                // WALLETS COLUMN
                div {
                    h2 {
                        style: "color: #00ff9f; margin-bottom: 30px; font-size: 20px; text-transform: uppercase; letter-spacing: 3px; border-bottom: 1px solid #00ff9f; padding-bottom: 15px; text-shadow: 0 0 10px rgba(0, 255, 159, 0.5);",
                        "// WALLET SYSTEM //"
                    }

                    // Wallet Health Status
                    div {
                        style: "margin: 0 0 20px 0; padding: 25px; border: 1px solid #fff; background: linear-gradient(135deg, #111 0%, #0a0a0a 100%); position: relative;",

                        div {
                            style: "position: absolute; top: 0; left: 0; right: 0; height: 2px; background: linear-gradient(90deg, transparent, #fff, transparent); opacity: 0.5;"
                        }

                        h3 {
                            style: "color: #fff; margin: 0 0 20px 0; font-size: 14px; text-transform: uppercase; letter-spacing: 3px;",
                            "// SYSTEM HEALTH //"
                        }

                        match health() {
                            Some(Ok(health_data)) => rsx! {
                                HealthStatus { health: health_data }
                            },
                            Some(Err(e)) => rsx! {
                                p {
                                    class: "error",
                                    style: "font-family: 'Courier New', monospace; font-size: 12px;",
                                    "ERROR: {e}"
                                }
                            },
                            None => rsx! {
                                HealthStatusSkeleton {}
                            }
                        }
                    }

                    // Wallet Balances
                    div {
                        style: "padding: 25px; border: 1px solid #fff; background: linear-gradient(135deg, #111 0%, #0a0a0a 100%); position: relative;",

                        div {
                            style: "position: absolute; top: 0; left: 0; right: 0; height: 2px; background: linear-gradient(90deg, transparent, #fff, transparent); opacity: 0.5;"
                        }

                        h3 {
                            style: "color: #fff; margin: 0 0 20px 0; font-size: 14px; text-transform: uppercase; letter-spacing: 3px;",
                            "// BALANCES //"
                        }

                        match balances() {
                            Some(Ok(balance_data)) => rsx! {
                                BalanceDisplay { balances: balance_data }
                            },
                            Some(Err(e)) => rsx! {
                                div {
                                    class: "error",
                                    "Backend Connection Error"
                                }
                                p {
                                    style: "font-family: 'Courier New', monospace; font-size: 11px; color: #666; margin-top: 10px;",
                                    "Unable to fetch wallet balances. Please check that the backend server is running on http://nixlab:3000"
                                }
                                details {
                                    summary {
                                        style: "color: #ff6b35; cursor: pointer; font-size: 11px; margin-top: 10px;",
                                        "Technical Details"
                                    }
                                    p {
                                        style: "font-family: 'Courier New', monospace; font-size: 10px; color: #999; margin-top: 5px;",
                                        "{e}"
                                    }
                                }
                            },
                            None => rsx! {
                                BalanceDisplaySkeleton {}
                            }
                        }
                    }
                }

                // TRADING COLUMN
                div {
                    h2 {
                        style: "color: #00d4ff; margin-bottom: 30px; font-size: 20px; text-transform: uppercase; letter-spacing: 3px; border-bottom: 1px solid #00d4ff; padding-bottom: 15px; text-shadow: 0 0 10px rgba(0, 212, 255, 0.5);",
                        "// TRADING ENGINE //"
                    }

                    // Trading Status
                    div {
                        style: "margin: 0 0 20px 0; padding: 25px; border: 1px solid #fff; background: linear-gradient(135deg, #111 0%, #0a0a0a 100%); position: relative;",

                        div {
                            style: "position: absolute; top: 0; left: 0; right: 0; height: 2px; background: linear-gradient(90deg, transparent, #fff, transparent); opacity: 0.5;"
                        }

                        h3 {
                            style: "color: #fff; margin: 0 0 20px 0; font-size: 14px; text-transform: uppercase; letter-spacing: 3px;",
                            "// ENGINE STATUS //"
                        }

                        match status() {
                            Some(Ok(status_data)) => rsx! {
                                StatusDisplay { status: status_data }
                            },
                            Some(Err(e)) => rsx! {
                                div {
                                    class: "error",
                                    "Backend Connection Error"
                                }
                                p {
                                    style: "font-family: 'Courier New', monospace; font-size: 11px; color: #666; margin-top: 10px;",
                                    "Unable to fetch trading engine status. Please check that the backend server is running."
                                }
                                details {
                                    summary {
                                        style: "color: #00d4ff; cursor: pointer; font-size: 11px; margin-top: 10px;",
                                        "Technical Details"
                                    }
                                    p {
                                        style: "font-family: 'Courier New', monospace; font-size: 10px; color: #999; margin-top: 5px;",
                                        "{e}"
                                    }
                                }
                            },
                            None => rsx! {
                                StatusDisplaySkeleton {}
                            }
                        }
                    }

                    // Trading Configuration
                    div {
                        style: "padding: 25px; border: 1px solid #fff; background: linear-gradient(135deg, #111 0%, #0a0a0a 100%); position: relative;",

                        div {
                            style: "position: absolute; top: 0; left: 0; right: 0; height: 2px; background: linear-gradient(90deg, transparent, #fff, transparent); opacity: 0.5;"
                        }

                        h3 {
                            style: "color: #fff; margin: 0 0 20px 0; font-size: 14px; text-transform: uppercase; letter-spacing: 3px;",
                            "// CONFIGURATION //"
                        }

                        match config() {
                            Some(Ok(config_data)) => rsx! {
                                ConfigDisplay { config: config_data }
                            },
                            Some(Err(e)) => rsx! {
                                div {
                                    class: "error",
                                    "Backend Connection Error"
                                }
                                p {
                                    style: "font-family: 'Courier New', monospace; font-size: 11px; color: #666; margin-top: 10px;",
                                    "Unable to fetch trading configuration. Please check that the backend server is running."
                                }
                                details {
                                    summary {
                                        style: "color: #ff00ff; cursor: pointer; font-size: 11px; margin-top: 10px;",
                                        "Technical Details"
                                    }
                                    p {
                                        style: "font-family: 'Courier New', monospace; font-size: 10px; color: #999; margin-top: 5px;",
                                        "{e}"
                                    }
                                }
                            },
                            None => rsx! {
                                ConfigDisplaySkeleton {}
                            }
                        }
                    }
                }
            }

            // METRICS SECTION
            div {
                h2 {
                    style: "color: #ff00ff; margin-bottom: 30px; font-size: 20px; text-transform: uppercase; letter-spacing: 3px; border-bottom: 1px solid #ff00ff; padding-bottom: 15px; text-shadow: 0 0 10px rgba(255, 0, 255, 0.5);",
                    "// SYSTEM METRICS //"
                }

                // Controls Panel
                div {
                    style: "margin: 0 0 40px 0; padding: 25px; border: 1px solid #333; background: linear-gradient(135deg, #111 0%, #0a0a0a 100%); position: relative;",

                    div {
                        style: "position: absolute; top: 0; left: 0; right: 0; height: 2px; background: linear-gradient(90deg, transparent, #fff, transparent); opacity: 0.3;"
                    }

                    div {
                        style: "display: flex; align-items: center; gap: 20px; margin-bottom: 25px; flex-wrap: wrap;",
                        label {
                            style: "color: #fff; text-transform: uppercase; font-size: 12px; letter-spacing: 2px;",
                            "// TIME INTERVAL:"
                        }
                        select {
                            value: "{interval}",
                            style: "padding: 10px 35px 10px 15px; border: 1px solid #333; background: #0a0a0a; color: #fff; font-family: 'Courier New', monospace; font-size: 12px; text-transform: uppercase; cursor: pointer;",
                            onchange: move |evt| {
                                if let Ok(val) = evt.value().parse::<i64>() {
                                    interval.set(val);
                                }
                            },
                            option { value: "5", "05 MIN" }
                            option { value: "15", "15 MIN" }
                            option { value: "30", "30 MIN" }
                            option { value: "60", "01 HOUR" }
                            option { value: "360", "06 HOURS" }
                            option { value: "1440", "24 HOURS" }
                        }
                    }

                    div {
                        style: "border-top: 1px solid #333; padding-top: 20px;",
                        h3 {
                            style: "color: #fff; margin: 0 0 15px 0; font-size: 12px; text-transform: uppercase; letter-spacing: 2px;",
                            "// ACTIVE MODULES:"
                        }
                        div {
                            style: "display: flex; gap: 25px; flex-wrap: wrap;",
                            label {
                                style: "display: flex; align-items: center; gap: 10px; color: #b0b0b0; cursor: pointer; padding: 8px 15px; border: 1px solid #333; background: #111; transition: all 0.3s ease;",
                                input { r#type: "checkbox", checked: show_bitcoin(), onchange: move |evt| show_bitcoin.set(evt.checked()) }
                                span { style: "text-transform: uppercase; font-size: 11px; letter-spacing: 1px;", "[ BTC ]" }
                            }
                            label {
                                style: "display: flex; align-items: center; gap: 10px; color: #b0b0b0; cursor: pointer; padding: 8px 15px; border: 1px solid #333; background: #111; transition: all 0.3s ease;",
                                input { r#type: "checkbox", checked: show_monero(), onchange: move |evt| show_monero.set(evt.checked()) }
                                span { style: "text-transform: uppercase; font-size: 11px; letter-spacing: 1px;", "[ XMR ]" }
                            }
                            label {
                                style: "display: flex; align-items: center; gap: 10px; color: #b0b0b0; cursor: pointer; padding: 8px 15px; border: 1px solid #333; background: #111; transition: all 0.3s ease;",
                                input { r#type: "checkbox", checked: show_asb(), onchange: move |evt| show_asb.set(evt.checked()) }
                                span { style: "text-transform: uppercase; font-size: 11px; letter-spacing: 1px;", "[ ASB ]" }
                            }
                        }
                    }
                }

                // Metrics Cards Grid
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(500px, 1fr)); gap: 30px;",

                    // Bitcoin Metrics Card
                    if show_bitcoin() {
                        BitcoinMetricsSection { interval: interval }
                    }

                    // Monero Metrics Card
                    if show_monero() {
                        MoneroMetricsSection { interval: interval }
                    }

                    // ASB Metrics Card
                    if show_asb() {
                        AsbMetricsSection { interval: interval }
                    }
                }
            }
        }
    }
}

