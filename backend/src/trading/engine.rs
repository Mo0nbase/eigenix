use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tokio::time::{sleep, Duration};

use crate::services::kraken::KrakenClient;
use crate::wallets::{BitcoinWallet, MoneroWallet};

use super::config::SharedTradingConfig;

/// Current state of the trading engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradingState {
    /// Engine is disabled
    Disabled,
    /// Engine is idle, monitoring balances
    Monitoring,
    /// Currently depositing Bitcoin to Kraken
    DepositingBitcoin { amount: f64 },
    /// Waiting for Bitcoin deposit to confirm on Kraken
    WaitingForBitcoinDeposit { txid: String },
    /// Executing BTC->XMR trade on Kraken
    Trading { btc_amount: f64 },
    /// Waiting for trade order to complete
    WaitingForTradeExecution { order_id: String },
    /// Withdrawing Monero from Kraken
    WithdrawingMonero { amount: f64 },
    /// Waiting for Monero withdrawal to complete
    WaitingForMoneroWithdrawal { refid: String },
    /// Error occurred during operation
    Error { message: String },
}

/// Status information about the trading engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingStatus {
    pub state: TradingState,
    pub enabled: bool,
    pub last_check: Option<String>,
    pub last_rebalance: Option<String>,
    pub current_btc_balance: Option<f64>,
    pub current_xmr_balance: Option<f64>,
    pub kraken_btc_balance: Option<f64>,
    pub kraken_xmr_balance: Option<f64>,
}

/// Thread-safe trading engine
#[derive(Clone)]
pub struct TradingEngine {
    pub config: SharedTradingConfig,
    state: Arc<RwLock<TradingState>>,
    enabled: Arc<RwLock<bool>>,
    kraken_api_key: String,
    kraken_api_secret: String,
    bitcoin_wallet_url: String,
    bitcoin_wallet_cookie: String,
    bitcoin_wallet_name: String,
    monero_wallet_url: String,
    monero_wallet_name: String,
    monero_wallet_password: String,
}

impl TradingEngine {
    /// Create a new trading engine
    pub fn new(
        config: SharedTradingConfig,
        kraken_api_key: String,
        kraken_api_secret: String,
        bitcoin_wallet_url: String,
        bitcoin_wallet_cookie: String,
        bitcoin_wallet_name: String,
        monero_wallet_url: String,
        monero_wallet_name: String,
        monero_wallet_password: String,
    ) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(TradingState::Disabled)),
            enabled: Arc::new(RwLock::new(false)),
            kraken_api_key,
            kraken_api_secret,
            bitcoin_wallet_url,
            bitcoin_wallet_cookie,
            bitcoin_wallet_name,
            monero_wallet_url,
            monero_wallet_name,
            monero_wallet_password,
        }
    }

    /// Enable the trading engine
    pub fn enable(&self) {
        *self.enabled.write().unwrap() = true;
        *self.state.write().unwrap() = TradingState::Monitoring;
        log::info!("Trading engine enabled");
    }

    /// Disable the trading engine
    pub fn disable(&self) {
        *self.enabled.write().unwrap() = false;
        *self.state.write().unwrap() = TradingState::Disabled;
        log::info!("Trading engine disabled");
    }

    /// Check if the trading engine is enabled
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read().unwrap()
    }

    /// Get the current state
    pub fn get_state(&self) -> TradingState {
        self.state.read().unwrap().clone()
    }

    /// Set the current state
    fn set_state(&self, state: TradingState) {
        *self.state.write().unwrap() = state;
    }

    /// Get trading status with balance information
    pub async fn get_status(&self) -> TradingStatus {
        let (btc_balance, xmr_balance) = self.get_wallet_balances().await.unwrap_or((None, None));
        let (kraken_btc, kraken_xmr) = self.get_kraken_balances().await.unwrap_or((None, None));

        TradingStatus {
            state: self.get_state(),
            enabled: self.is_enabled(),
            last_check: None,    // TODO: Track this
            last_rebalance: None, // TODO: Track this
            current_btc_balance: btc_balance,
            current_xmr_balance: xmr_balance,
            kraken_btc_balance: kraken_btc,
            kraken_xmr_balance: kraken_xmr,
        }
    }

    /// Main trading loop
    pub async fn run(self) {
        log::info!("Trading engine started");

        loop {
            if !self.is_enabled() {
                // Sleep for a while when disabled
                sleep(Duration::from_secs(10)).await;
                continue;
            }

            let config = self.config.get();

            // Run one iteration of the trading logic
            if let Err(e) = self.check_and_rebalance().await {
                log::error!("Trading engine error: {}", e);
                self.set_state(TradingState::Error {
                    message: e.to_string(),
                });
                // Wait a bit before retrying after error
                sleep(Duration::from_secs(60)).await;
                continue;
            }

            // Sleep until next check
            sleep(Duration::from_secs(config.check_interval_secs)).await;
        }
    }

    /// Check balances and rebalance if needed
    async fn check_and_rebalance(&self) -> Result<()> {
        self.set_state(TradingState::Monitoring);

        let config = self.config.get();

        // Get current balances
        let (btc_balance, xmr_balance) = self.get_wallet_balances().await?;
        
        let btc_balance = btc_balance.context("Bitcoin balance not available")?;
        let xmr_balance = xmr_balance.context("Monero balance not available")?;

        log::info!(
            "Current balances - BTC: {:.8}, XMR: {:.8}",
            btc_balance,
            xmr_balance
        );

        // Check if rebalancing is needed
        if xmr_balance >= config.monero_min_threshold {
            log::debug!("XMR balance ({:.8}) above threshold ({:.8}), no rebalancing needed",
                xmr_balance, config.monero_min_threshold);
            return Ok(());
        }

        log::info!(
            "XMR balance ({:.8}) below threshold ({:.8}), starting rebalancing",
            xmr_balance,
            config.monero_min_threshold
        );

        // Calculate how much XMR we need
        let xmr_needed = config.monero_target_balance - xmr_balance;
        log::info!("Need to acquire {:.8} XMR", xmr_needed);

        // Execute the rebalancing workflow
        self.execute_rebalance(xmr_needed).await?;

        Ok(())
    }

    /// Execute the full rebalancing workflow
    async fn execute_rebalance(&self, xmr_needed: f64) -> Result<()> {
        let config = self.config.get();

        // Step 1: Get current BTC/XMR price from Kraken
        let kraken = KrakenClient::new(
            self.kraken_api_key.clone(),
            self.kraken_api_secret.clone(),
        );

        let ticker = kraken.get_ticker("XBTXMR").await
            .context("Failed to get BTC/XMR ticker from Kraken")?;
        
        let btc_xmr_price: f64 = ticker.last_trade[0].parse()
            .context("Failed to parse BTC/XMR price")?;
        
        log::info!("Current BTC/XMR rate: {:.8}", btc_xmr_price);

        // Calculate how much BTC we need (with slippage buffer)
        let slippage_multiplier = 1.0 + (config.slippage_tolerance_percent / 100.0);
        let btc_needed = xmr_needed * btc_xmr_price * slippage_multiplier;
        
        // Cap at max BTC per rebalance
        let btc_to_use = btc_needed.min(config.max_btc_per_rebalance);
        
        log::info!("Will use {:.8} BTC for rebalancing", btc_to_use);

        // Check if we have enough BTC (keeping reserve)
        let (btc_balance, _) = self.get_wallet_balances().await?;
        let btc_balance = btc_balance.context("Bitcoin balance not available")?;
        
        let btc_available = btc_balance - config.bitcoin_reserve_minimum;
        if btc_available < btc_to_use {
            anyhow::bail!(
                "Insufficient BTC: need {:.8}, have {:.8} available (after reserve)",
                btc_to_use, btc_available
            );
        }

        // Step 2: Deposit BTC to Kraken
        log::info!("Step 1: Depositing {:.8} BTC to Kraken", btc_to_use);
        let btc_txid = self.deposit_bitcoin_to_kraken(btc_to_use).await?;
        
        // Step 3: Wait for deposit to confirm
        log::info!("Step 2: Waiting for BTC deposit to confirm (txid: {})", btc_txid);
        self.wait_for_bitcoin_deposit(&kraken, &btc_txid).await?;

        // Step 4: Execute BTC->XMR trade on Kraken
        log::info!("Step 3: Executing BTC->XMR trade on Kraken");
        let order_id = self.execute_btc_to_xmr_trade(&kraken, btc_to_use, &config).await?;

        // Step 5: Wait for trade to execute
        log::info!("Step 4: Waiting for trade execution (order: {})", order_id);
        let xmr_amount = self.wait_for_trade_execution(&kraken, &order_id, &config).await?;

        // Step 6: Withdraw XMR from Kraken
        log::info!("Step 5: Withdrawing {:.8} XMR from Kraken", xmr_amount);
        let withdraw_refid = self.withdraw_monero_from_kraken(&kraken, xmr_amount).await?;

        // Step 7: Wait for withdrawal to complete
        log::info!("Step 6: Waiting for XMR withdrawal (refid: {})", withdraw_refid);
        self.wait_for_monero_withdrawal(&kraken, &withdraw_refid).await?;

        log::info!("Rebalancing completed successfully!");
        Ok(())
    }

    /// Get wallet balances (BTC, XMR)
    async fn get_wallet_balances(&self) -> Result<(Option<f64>, Option<f64>)> {
        let btc_balance = match BitcoinWallet::connect_existing(
            self.bitcoin_wallet_url.clone(),
            &self.bitcoin_wallet_cookie,
            &self.bitcoin_wallet_name,
        )
        .await
        {
            Ok(wallet) => match wallet.get_balance().await {
                Ok(balance) => Some(balance.balance),
                Err(_) => None,
            },
            Err(_) => None,
        };

        let xmr_balance = match MoneroWallet::connect_existing(
            self.monero_wallet_url.clone(),
            &self.monero_wallet_name,
            &self.monero_wallet_password,
        )
        .await
        {
            Ok(wallet) => match wallet.get_balance().await {
                Ok(balance) => Some(balance.unlocked_balance),
                Err(_) => None,
            },
            Err(_) => None,
        };

        Ok((btc_balance, xmr_balance))
    }

    /// Get Kraken balances (BTC, XMR)
    async fn get_kraken_balances(&self) -> Result<(Option<f64>, Option<f64>)> {
        let kraken = KrakenClient::new(
            self.kraken_api_key.clone(),
            self.kraken_api_secret.clone(),
        );

        let balances = kraken.get_balance().await?;
        
        let btc = balances.get("XXBT").and_then(|s| s.parse::<f64>().ok());
        let xmr = balances.get("XXMR").and_then(|s| s.parse::<f64>().ok());

        Ok((btc, xmr))
    }

    /// Deposit Bitcoin to Kraken
    async fn deposit_bitcoin_to_kraken(&self, amount: f64) -> Result<String> {
        self.set_state(TradingState::DepositingBitcoin { amount });

        let kraken = KrakenClient::new(
            self.kraken_api_key.clone(),
            self.kraken_api_secret.clone(),
        );

        // Get Kraken BTC deposit address
        let deposit_address = kraken.get_btc_deposit_address(false).await
            .context("Failed to get Kraken BTC deposit address")?;

        log::info!("Kraken BTC deposit address: {}", deposit_address);

        // Send BTC from our wallet to Kraken
        let btc_wallet = BitcoinWallet::connect_existing(
            self.bitcoin_wallet_url.clone(),
            &self.bitcoin_wallet_cookie,
            &self.bitcoin_wallet_name,
        )
        .await
        .context("Failed to connect to Bitcoin wallet")?;

        let txid = btc_wallet.send(&deposit_address, amount).await
            .context("Failed to send Bitcoin to Kraken")?;

        log::info!("Bitcoin sent to Kraken, txid: {}", txid);
        self.set_state(TradingState::WaitingForBitcoinDeposit { txid: txid.clone() });

        Ok(txid)
    }

    /// Wait for Bitcoin deposit to confirm on Kraken
    async fn wait_for_bitcoin_deposit(&self, kraken: &KrakenClient, _txid: &str) -> Result<()> {
        // Poll deposit status until confirmed
        let timeout = Duration::from_secs(3600); // 1 hour timeout
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                anyhow::bail!("Timeout waiting for Bitcoin deposit confirmation");
            }

            let deposits = kraken.get_deposit_status(Some("XBT")).await?;
            
            // Check if we have a recent confirmed deposit
            // Note: This is simplified - in production you'd want to match the specific txid
            if let Some(deposit) = deposits.first() {
                if deposit.status == "Success" {
                    log::info!("Bitcoin deposit confirmed on Kraken");
                    return Ok(());
                }
            }

            log::debug!("Waiting for Bitcoin deposit confirmation...");
            sleep(Duration::from_secs(30)).await;
        }
    }

    /// Execute BTC->XMR trade on Kraken
    async fn execute_btc_to_xmr_trade(
        &self,
        kraken: &KrakenClient,
        btc_amount: f64,
        config: &crate::trading::config::TradingConfig,
    ) -> Result<String> {
        self.set_state(TradingState::Trading { btc_amount });

        let order_type = if config.use_limit_orders {
            "limit"
        } else {
            "market"
        };

        // For limit orders, calculate a price with slippage tolerance
        let price = if config.use_limit_orders {
            let ticker = kraken.get_ticker("XBTXMR").await?;
            let current_price: f64 = ticker.ask[0].parse()?;
            let price_with_slippage = current_price * (1.0 + config.slippage_tolerance_percent / 100.0);
            Some(format!("{:.8}", price_with_slippage))
        } else {
            None
        };

        let order = kraken
            .place_order(
                "XBTXMR",
                "buy",
                order_type,
                &format!("{:.8}", btc_amount),
                price.as_deref(),
            )
            .await
            .context("Failed to place order on Kraken")?;

        let order_id = order.txid.first()
            .context("No order ID returned from Kraken")?
            .clone();

        log::info!("Order placed on Kraken: {}", order_id);
        self.set_state(TradingState::WaitingForTradeExecution { order_id: order_id.clone() });

        Ok(order_id)
    }

    /// Wait for trade to execute
    async fn wait_for_trade_execution(
        &self,
        kraken: &KrakenClient,
        order_id: &str,
        config: &crate::trading::config::TradingConfig,
    ) -> Result<f64> {
        let timeout = Duration::from_secs(config.order_timeout_secs);
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                anyhow::bail!("Timeout waiting for order execution");
            }

            let order_status = kraken.query_order(order_id).await?;
            
            if let Some(order_info) = order_status.get(order_id) {
                let status = order_info.get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if status == "closed" {
                    // Order executed successfully
                    let vol_exec = order_info.get("vol_exec")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .context("Failed to parse executed volume")?;

                    log::info!("Trade executed successfully, received {:.8} XMR", vol_exec);
                    return Ok(vol_exec);
                } else if status == "canceled" || status == "expired" {
                    anyhow::bail!("Order was canceled or expired");
                }
            }

            log::debug!("Waiting for order execution...");
            sleep(Duration::from_secs(10)).await;
        }
    }

    /// Withdraw Monero from Kraken
    async fn withdraw_monero_from_kraken(&self, kraken: &KrakenClient, amount: f64) -> Result<String> {
        self.set_state(TradingState::WithdrawingMonero { amount });

        // Get our Monero wallet address
        let xmr_wallet = MoneroWallet::connect_existing(
            self.monero_wallet_url.clone(),
            &self.monero_wallet_name,
            &self.monero_wallet_password,
        )
        .await
        .context("Failed to connect to Monero wallet")?;

        let address = xmr_wallet.get_address(0).await
            .context("Failed to get Monero address")?;

        log::info!("Withdrawing to Monero address: {}", address.address);

        // Initiate withdrawal from Kraken
        let withdraw_result = kraken.withdraw_xmr(&address.address, amount).await
            .context("Failed to initiate Monero withdrawal from Kraken")?;

        let refid = withdraw_result.refid;
        log::info!("Monero withdrawal initiated: {}", refid);
        self.set_state(TradingState::WaitingForMoneroWithdrawal { refid: refid.clone() });

        Ok(refid)
    }

    /// Wait for Monero withdrawal to complete
    async fn wait_for_monero_withdrawal(&self, kraken: &KrakenClient, refid: &str) -> Result<()> {
        let timeout = Duration::from_secs(3600); // 1 hour timeout
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                anyhow::bail!("Timeout waiting for Monero withdrawal");
            }

            let withdrawals = kraken.get_withdrawal_status(Some("XMR")).await?;
            
            // Find our withdrawal
            if let Some(withdrawal) = withdrawals.iter().find(|w| w.refid == refid) {
                if withdrawal.status == "Success" {
                    log::info!("Monero withdrawal completed successfully");
                    return Ok(());
                } else if withdrawal.status == "Failure" || withdrawal.status == "Canceled" {
                    anyhow::bail!("Monero withdrawal failed or was canceled");
                }
            }

            log::debug!("Waiting for Monero withdrawal completion...");
            sleep(Duration::from_secs(30)).await;
        }
    }
}

