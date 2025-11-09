use dioxus::prelude::*;

use crate::api;
use crate::components::{CharmingChart, Navbar};
use crate::types::metrics::{
    AsbMetrics, BitcoinMetrics, MetricValue, MoneroMetrics, TradingConfig, TradingStatus,
    WalletBalances, WalletHealth,
};

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
                                p {
                                    class: "loading",
                                    style: "font-family: 'Courier New', monospace; font-size: 12px;",
                                    "// LOADING HEALTH STATUS..."
                                }
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
                                p {
                                    class: "error",
                                    style: "font-family: 'Courier New', monospace; font-size: 12px;",
                                    "ERROR: {e}"
                                }
                            },
                            None => rsx! {
                                p {
                                    class: "loading",
                                    style: "font-family: 'Courier New', monospace; font-size: 12px;",
                                    "// LOADING BALANCES..."
                                }
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
                                p {
                                    class: "error",
                                    style: "font-family: 'Courier New', monospace; font-size: 12px;",
                                    "ERROR: {e}"
                                }
                            },
                            None => rsx! {
                                p {
                                    class: "loading",
                                    style: "font-family: 'Courier New', monospace; font-size: 12px;",
                                    "// LOADING STATUS..."
                                }
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
                                p {
                                    class: "error",
                                    style: "font-family: 'Courier New', monospace; font-size: 12px;",
                                    "ERROR: {e}"
                                }
                            },
                            None => rsx! {
                                p {
                                    class: "loading",
                                    style: "font-family: 'Courier New', monospace; font-size: 12px;",
                                    "// LOADING CONFIGURATION..."
                                }
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

// ===== METRICS COMPONENTS =====

#[component]
fn BitcoinMetricsSection(interval: Signal<i64>) -> Element {
    let data = use_resource(move || async move {
        api::metrics::fetch_bitcoin_interval(interval()).await
    });

    rsx! {
        div {
            style: "padding: 25px; border: 1px solid #333; background: linear-gradient(135deg, #0a0a0a 0%, #111 100%); position: relative;",

            div {
                style: "position: absolute; top: 0; left: 0; right: 0; height: 2px; background: linear-gradient(90deg, transparent, #fff, transparent); opacity: 0.3;"
            }

            h3 {
                style: "color: #ffa500; margin: 0 0 25px 0; font-size: 16px; text-transform: uppercase; letter-spacing: 3px; border-bottom: 1px solid #ffa500; padding-bottom: 15px; text-shadow: 0 0 10px rgba(255, 165, 0, 0.5);",
                "// BTC NODE //"
            }

            match data() {
                Some(Ok(metrics)) => rsx! {
                    BitcoinCharts { data: metrics }
                },
                Some(Err(e)) => rsx! {
                    p {
                        class: "error",
                        style: "font-family: 'Courier New', monospace; font-size: 12px;",
                        "ERROR: {e}"
                    }
                },
                None => rsx! {
                    p {
                        class: "loading",
                        style: "font-family: 'Courier New', monospace; font-size: 12px;",
                        "// LOADING BTC METRICS..."
                    }
                }
            }
        }
    }
}

#[component]
fn BitcoinCharts(data: Vec<BitcoinMetrics>) -> Element {
    let blocks_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.blocks as f64,
        })
        .collect();

    let progress_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.verification_progress * 100.0,
        })
        .collect();

    let balance_data: Vec<MetricValue> = data
        .iter()
        .filter_map(|m| {
            m.wallet_balance.map(|b| MetricValue {
                timestamp: m.timestamp.clone(),
                value: b,
            })
        })
        .collect();

    rsx! {
        CharmingChart {
            id: "btc-blocks".to_string(),
            title: "BLOCK HEIGHT".to_string(),
            data: blocks_data,
            color: "#ffa500".to_string(),
            y_begin_at_zero: false
        }
        CharmingChart {
            id: "btc-progress".to_string(),
            title: "SYNC PROGRESS %".to_string(),
            data: progress_data,
            color: "#ffa500".to_string(),
            y_begin_at_zero: true
        }
        if !balance_data.is_empty() {
            CharmingChart {
                id: "btc-balance".to_string(),
                title: "WALLET BALANCE BTC".to_string(),
                data: balance_data,
                color: "#ffa500".to_string(),
                y_begin_at_zero: false
            }
        }
    }
}

#[component]
fn MoneroMetricsSection(interval: Signal<i64>) -> Element {
    let data = use_resource(move || async move {
        api::metrics::fetch_monero_interval(interval()).await
    });

    rsx! {
        div {
            style: "padding: 25px; border: 1px solid #333; background: linear-gradient(135deg, #0a0a0a 0%, #111 100%); position: relative;",

            div {
                style: "position: absolute; top: 0; left: 0; right: 0; height: 2px; background: linear-gradient(90deg, transparent, #fff, transparent); opacity: 0.3;"
            }

            h3 {
                style: "color: #ff6b35; margin: 0 0 25px 0; font-size: 16px; text-transform: uppercase; letter-spacing: 3px; border-bottom: 1px solid #ff6b35; padding-bottom: 15px; text-shadow: 0 0 10px rgba(255, 107, 53, 0.5);",
                "// XMR NODE //"
            }

            match data() {
                Some(Ok(metrics)) => rsx! {
                    MoneroCharts { data: metrics }
                },
                Some(Err(e)) => rsx! {
                    p {
                        class: "error",
                        style: "font-family: 'Courier New', monospace; font-size: 12px;",
                        "ERROR: {e}"
                    }
                },
                None => rsx! {
                    p {
                        class: "loading",
                        style: "font-family: 'Courier New', monospace; font-size: 12px;",
                        "// LOADING XMR METRICS..."
                    }
                }
            }
        }
    }
}

#[component]
fn MoneroCharts(data: Vec<MoneroMetrics>) -> Element {
    let height_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.height as f64,
        })
        .collect();

    let difficulty_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.difficulty as f64,
        })
        .collect();

    let tx_count_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.tx_count as f64,
        })
        .collect();

    let balance_data: Vec<MetricValue> = data
        .iter()
        .filter_map(|m| {
            m.wallet_balance.map(|b| MetricValue {
                timestamp: m.timestamp.clone(),
                value: b,
            })
        })
        .collect();

    rsx! {
        CharmingChart {
            id: "xmr-height".to_string(),
            title: "BLOCK HEIGHT".to_string(),
            data: height_data,
            color: "#ff6b35".to_string(),
            y_begin_at_zero: false
        }
        CharmingChart {
            id: "xmr-difficulty".to_string(),
            title: "NETWORK DIFFICULTY".to_string(),
            data: difficulty_data,
            color: "#ff6b35".to_string(),
            y_begin_at_zero: false
        }
        CharmingChart {
            id: "xmr-txcount".to_string(),
            title: "TRANSACTION COUNT".to_string(),
            data: tx_count_data,
            color: "#ff6b35".to_string(),
            y_begin_at_zero: false
        }
        if !balance_data.is_empty() {
            CharmingChart {
                id: "xmr-balance".to_string(),
                title: "WALLET BALANCE XMR".to_string(),
                data: balance_data,
                color: "#ff6b35".to_string(),
                y_begin_at_zero: false
            }
        }
    }
}

#[component]
fn AsbMetricsSection(interval: Signal<i64>) -> Element {
    let data =
        use_resource(move || async move { api::metrics::fetch_asb_interval(interval()).await });

    rsx! {
        div {
            style: "padding: 25px; border: 1px solid #333; background: linear-gradient(135deg, #0a0a0a 0%, #111 100%); position: relative;",

            div {
                style: "position: absolute; top: 0; left: 0; right: 0; height: 2px; background: linear-gradient(90deg, transparent, #fff, transparent); opacity: 0.3;"
            }

            h3 {
                style: "color: #00d4ff; margin: 0 0 25px 0; font-size: 16px; text-transform: uppercase; letter-spacing: 3px; border-bottom: 1px solid #00d4ff; padding-bottom: 15px; text-shadow: 0 0 10px rgba(0, 212, 255, 0.5);",
                "// ASB STATUS //"
            }

            match data() {
                Some(Ok(metrics)) => rsx! {
                    AsbCharts { data: metrics }
                },
                Some(Err(e)) => rsx! {
                    p {
                        class: "error",
                        style: "font-family: 'Courier New', monospace; font-size: 12px;",
                        "ERROR: {e}"
                    }
                },
                None => rsx! {
                    p {
                        class: "loading",
                        style: "font-family: 'Courier New', monospace; font-size: 12px;",
                        "// LOADING ASB METRICS..."
                    }
                }
            }
        }
    }
}

#[component]
fn AsbCharts(data: Vec<AsbMetrics>) -> Element {
    let balance_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.balance_btc,
        })
        .collect();

    let pending_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.pending_swaps as f64,
        })
        .collect();

    let completed_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.completed_swaps as f64,
        })
        .collect();

    let failed_data: Vec<MetricValue> = data
        .iter()
        .map(|m| MetricValue {
            timestamp: m.timestamp.clone(),
            value: m.failed_swaps as f64,
        })
        .collect();

    rsx! {
        CharmingChart {
            id: "asb-balance".to_string(),
            title: "BTC BALANCE".to_string(),
            data: balance_data,
            color: "#00d4ff".to_string(),
            y_begin_at_zero: false
        }
        CharmingChart {
            id: "asb-pending".to_string(),
            title: "PENDING SWAPS".to_string(),
            data: pending_data,
            color: "#ffff00".to_string(),
            y_begin_at_zero: true
        }
        CharmingChart {
            id: "asb-completed".to_string(),
            title: "COMPLETED SWAPS".to_string(),
            data: completed_data,
            color: "#00ff9f".to_string(),
            y_begin_at_zero: true
        }
        CharmingChart {
            id: "asb-failed".to_string(),
            title: "FAILED SWAPS".to_string(),
            data: failed_data,
            color: "#ff3333".to_string(),
            y_begin_at_zero: true
        }
    }
}

// ===== WALLETS COMPONENTS =====

#[component]
fn HealthStatus(health: WalletHealth) -> Element {
    let overall_status = if health.healthy { "ONLINE" } else { "OFFLINE" };
    let overall_color = if health.healthy { "#00ff9f" } else { "#ff3333" };
    let btc_status = if health.bitcoin_ready { "READY" } else { "NOT READY" };
    let btc_color = if health.bitcoin_ready { "#00ff9f" } else { "#ff3333" };
    let xmr_status = if health.monero_ready { "READY" } else { "NOT READY" };
    let xmr_color = if health.monero_ready { "#00ff9f" } else { "#ff3333" };

    rsx! {
        div {
            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px;",

            div {
                style: "padding: 20px; background: #0a0a0a; border: 1px solid #333; position: relative; overflow: hidden;",

                div {
                    style: "position: absolute; top: 0; left: 0; width: 4px; height: 100%; background: {overall_color}; box-shadow: 0 0 10px {overall_color};"
                }

                h4 {
                    style: "color: #b0b0b0; margin: 0 0 10px 0; font-size: 10px; letter-spacing: 2px; text-transform: uppercase;",
                    "OVERALL STATUS"
                }
                p {
                    style: "color: {overall_color}; margin: 0; font-size: 20px; font-weight: bold; letter-spacing: 2px; text-shadow: 0 0 10px {overall_color};",
                    "{overall_status}"
                }
            }

            div {
                style: "padding: 20px; background: #0a0a0a; border: 1px solid #333; position: relative; overflow: hidden;",

                div {
                    style: "position: absolute; top: 0; left: 0; width: 4px; height: 100%; background: {btc_color}; box-shadow: 0 0 10px {btc_color};"
                }

                h4 {
                    style: "color: #b0b0b0; margin: 0 0 10px 0; font-size: 10px; letter-spacing: 2px; text-transform: uppercase;",
                    "[ BTC ] STATUS"
                }
                p {
                    style: "color: {btc_color}; margin: 0; font-size: 16px; font-weight: bold; letter-spacing: 1px; text-shadow: 0 0 10px {btc_color};",
                    "{btc_status}"
                }
            }

            div {
                style: "padding: 20px; background: #0a0a0a; border: 1px solid #333; position: relative; overflow: hidden;",

                div {
                    style: "position: absolute; top: 0; left: 0; width: 4px; height: 100%; background: {xmr_color}; box-shadow: 0 0 10px {xmr_color};"
                }

                h4 {
                    style: "color: #b0b0b0; margin: 0 0 10px 0; font-size: 10px; letter-spacing: 2px; text-transform: uppercase;",
                    "[ XMR ] STATUS"
                }
                p {
                    style: "color: {xmr_color}; margin: 0; font-size: 16px; font-weight: bold; letter-spacing: 1px; text-shadow: 0 0 10px {xmr_color};",
                    "{xmr_status}"
                }
            }
        }
    }
}

#[component]
fn BalanceDisplay(balances: WalletBalances) -> Element {
    rsx! {
        div {
            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(350px, 1fr)); gap: 30px;",

            div {
                style: "padding: 30px; background: linear-gradient(135deg, #0a0a0a 0%, #111 100%); border: 2px solid #ffa500; position: relative; overflow: visible;",

                div {
                    style: "position: absolute; top: 0; right: 0; width: 100px; height: 100px; background: radial-gradient(circle, rgba(255, 165, 0, 0.1) 0%, transparent 70%); pointer-events: none;"
                }

                h4 {
                    style: "color: #ffa500; margin: 0 0 20px 0; font-size: 12px; text-transform: uppercase; letter-spacing: 3px; text-shadow: 0 0 10px rgba(255, 165, 0, 0.5);",
                    "// BITCOIN //"
                }
                div {
                    style: "display: flex; align-items: baseline; gap: 8px; position: relative; z-index: 1;",
                    p {
                        style: "color: #ffa500; margin: 0; font-size: 36px; font-weight: bold; text-shadow: 0 0 15px rgba(255, 165, 0, 0.8); font-family: 'Courier New', monospace;",
                        "{balances.bitcoin:.8}"
                    }
                    span {
                        style: "color: #ffa500; font-size: 16px; text-transform: uppercase; letter-spacing: 2px; opacity: 0.7;",
                        "BTC"
                    }
                }
            }

            div {
                style: "padding: 30px; background: linear-gradient(135deg, #0a0a0a 0%, #111 100%); border: 2px solid #ff6b35; position: relative; overflow: visible;",

                div {
                    style: "position: absolute; top: 0; right: 0; width: 100px; height: 100px; background: radial-gradient(circle, rgba(255, 107, 53, 0.1) 0%, transparent 70%); pointer-events: none;"
                }

                h4 {
                    style: "color: #ff6b35; margin: 0 0 20px 0; font-size: 12px; text-transform: uppercase; letter-spacing: 3px; text-shadow: 0 0 10px rgba(255, 107, 53, 0.5);",
                    "// MONERO //"
                }
                div {
                    style: "display: flex; align-items: baseline; gap: 8px; flex-wrap: wrap; position: relative; z-index: 1;",
                    p {
                        style: "color: #ff6b35; margin: 0; font-size: 28px; font-weight: bold; text-shadow: 0 0 15px rgba(255, 107, 53, 0.8); font-family: 'Courier New', monospace; word-break: break-all;",
                        "{balances.monero:.12}"
                    }
                    span {
                        style: "color: #ff6b35; font-size: 16px; text-transform: uppercase; letter-spacing: 2px; opacity: 0.7;",
                        "XMR"
                    }
                }
            }
        }
    }
}

// ===== TRADING COMPONENTS =====

#[component]
fn StatusDisplay(status: TradingStatus) -> Element {
    let status_text = if status.enabled { "ENABLED" } else { "DISABLED" };
    let status_color = if status.enabled { "#00d4ff" } else { "#ff3333" };

    rsx! {
        div {
            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 20px;",

            div {
                style: "padding: 25px; background: #0a0a0a; border: 1px solid #333; position: relative; overflow: hidden;",

                div {
                    style: "position: absolute; top: 0; left: 0; width: 4px; height: 100%; background: {status_color}; box-shadow: 0 0 10px {status_color};"
                }

                h4 {
                    style: "color: #b0b0b0; margin: 0 0 15px 0; font-size: 10px; letter-spacing: 2px; text-transform: uppercase;",
                    "ENGINE STATE"
                }
                p {
                    style: "color: {status_color}; margin: 0; font-size: 28px; font-weight: bold; letter-spacing: 2px; text-shadow: 0 0 15px {status_color}; font-family: 'Courier New', monospace;",
                    "{status_text}"
                }
            }

            if let Some(last_check) = &status.last_check {
                div {
                    style: "padding: 25px; background: #0a0a0a; border: 1px solid #333; position: relative; overflow: hidden;",

                    div {
                        style: "position: absolute; top: 0; left: 0; width: 4px; height: 100%; background: #707070;"
                    }

                    h4 {
                        style: "color: #b0b0b0; margin: 0 0 15px 0; font-size: 10px; letter-spacing: 2px; text-transform: uppercase;",
                        "LAST CHECK"
                    }
                    p {
                        style: "color: #fff; margin: 0; font-size: 14px; font-family: 'Courier New', monospace;",
                        "{last_check}"
                    }
                }
            }

            if let Some(last_trade) = &status.last_trade {
                div {
                    style: "padding: 25px; background: #0a0a0a; border: 1px solid #333; position: relative; overflow: hidden;",

                    div {
                        style: "position: absolute; top: 0; left: 0; width: 4px; height: 100%; background: #00d4ff; box-shadow: 0 0 10px #00d4ff;"
                    }

                    h4 {
                        style: "color: #b0b0b0; margin: 0 0 15px 0; font-size: 10px; letter-spacing: 2px; text-transform: uppercase;",
                        "LAST TRADE"
                    }
                    p {
                        style: "color: #00d4ff; margin: 0; font-size: 14px; font-family: 'Courier New', monospace; text-shadow: 0 0 10px rgba(0, 212, 255, 0.5);",
                        "{last_trade}"
                    }
                }
            }
        }
    }
}

#[component]
fn ConfigDisplay(config: TradingConfig) -> Element {
    rsx! {
        div {
            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 20px;",

            div {
                style: "padding: 20px; background: #0a0a0a; border: 1px solid #333;",
                h5 {
                    style: "color: #b0b0b0; margin: 0 0 10px 0; font-size: 10px; letter-spacing: 2px; text-transform: uppercase;",
                    "TARGET BTC %"
                }
                p {
                    style: "color: #fff; margin: 0; font-size: 24px; font-weight: bold; font-family: 'Courier New', monospace;",
                    "{config.target_btc_percentage}%"
                }
            }

            div {
                style: "padding: 20px; background: #0a0a0a; border: 1px solid #333;",
                h5 {
                    style: "color: #b0b0b0; margin: 0 0 10px 0; font-size: 10px; letter-spacing: 2px; text-transform: uppercase;",
                    "REBALANCE THRESHOLD"
                }
                p {
                    style: "color: #fff; margin: 0; font-size: 24px; font-weight: bold; font-family: 'Courier New', monospace;",
                    "{config.rebalance_threshold_percentage}%"
                }
            }

            div {
                style: "padding: 20px; background: #0a0a0a; border: 1px solid #333;",
                h5 {
                    style: "color: #b0b0b0; margin: 0 0 10px 0; font-size: 10px; letter-spacing: 2px; text-transform: uppercase;",
                    "MAX TRADE SIZE"
                }
                p {
                    style: "color: #fff; margin: 0; font-size: 20px; font-weight: bold; font-family: 'Courier New', monospace;",
                    "{config.max_trade_size_btc} BTC"
                }
            }

            div {
                style: "padding: 20px; background: #0a0a0a; border: 1px solid #333;",
                h5 {
                    style: "color: #b0b0b0; margin: 0 0 10px 0; font-size: 10px; letter-spacing: 2px; text-transform: uppercase;",
                    "MIN TRADE SIZE"
                }
                p {
                    style: "color: #fff; margin: 0; font-size: 20px; font-weight: bold; font-family: 'Courier New', monospace;",
                    "{config.min_trade_size_btc} BTC"
                }
            }

            div {
                style: "padding: 20px; background: #0a0a0a; border: 1px solid #333;",
                h5 {
                    style: "color: #b0b0b0; margin: 0 0 10px 0; font-size: 10px; letter-spacing: 2px; text-transform: uppercase;",
                    "CHECK INTERVAL"
                }
                p {
                    style: "color: #fff; margin: 0; font-size: 20px; font-weight: bold; font-family: 'Courier New', monospace;",
                    "{config.check_interval_seconds}s"
                }
            }
        }
    }
}

