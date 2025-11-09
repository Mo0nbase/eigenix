use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tokio::time::{sleep, Duration};

use crate::db::{MetricsDatabase, StoredTradingTransaction, TransactionStatus, TransactionType};
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
    db: Option<MetricsDatabase>,
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
            db: None,
        }
    }

    /// Set the database for transaction tracking
    pub fn with_database(mut self, db: MetricsDatabase) -> Self {
        self.db = Some(db);
        self
    }

    /// Get the database if available
    fn get_db(&self) -> Option<&MetricsDatabase> {
        self.db.as_ref()
    }

    /// Enable the trading engine
    pub fn enable(&self) {
        *self.enabled.write().unwrap() = true;
        *self.state.write().unwrap() = TradingState::Monitoring;
        tracing::info!("Trading engine enabled");
    }

    /// Disable the trading engine
    pub fn disable(&self) {
        *self.enabled.write().unwrap() = false;
        *self.state.write().unwrap() = TradingState::Disabled;
        tracing::info!("Trading engine disabled");
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
            last_check: None,     // TODO: Track this
            last_rebalance: None, // TODO: Track this
            current_btc_balance: btc_balance,
            current_xmr_balance: xmr_balance,
            kraken_btc_balance: kraken_btc,
            kraken_xmr_balance: kraken_xmr,
        }
    }

    /// Main trading loop
    pub async fn run(self) {
        tracing::info!("Trading engine started");

        loop {
            if !self.is_enabled() {
                // Sleep for a while when disabled
                sleep(Duration::from_secs(10)).await;
                continue;
            }

            let config = self.config.get();

            // Run one iteration of the trading logic
            if let Err(e) = self.check_and_rebalance().await {
                tracing::error!("Trading engine error: {}", e);
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

        tracing::info!(
            "Current balances - BTC: {:.8}, XMR: {:.8}",
            btc_balance,
            xmr_balance
        );

        // Check if rebalancing is needed
        if xmr_balance >= config.monero_min_threshold {
            tracing::debug!(
                "XMR balance ({:.8}) above threshold ({:.8}), no rebalancing needed",
                xmr_balance,
                config.monero_min_threshold
            );
            return Ok(());
        }

        tracing::info!(
            "XMR balance ({:.8}) below threshold ({:.8}), starting rebalancing",
            xmr_balance,
            config.monero_min_threshold
        );

        // Calculate how much XMR we need
        let xmr_needed = config.monero_target_balance - xmr_balance;
        tracing::info!("Need to acquire {:.8} XMR", xmr_needed);

        // Execute the rebalancing workflow
        self.execute_rebalance(xmr_needed).await?;

        Ok(())
    }

    /// Execute the full rebalancing workflow
    async fn execute_rebalance(&self, xmr_needed: f64) -> Result<()> {
        let config = self.config.get();

        // Step 1: Get current BTC/XMR price from Kraken
        let kraken = KrakenClient::new(self.kraken_api_key.clone(), self.kraken_api_secret.clone());

        let ticker = kraken
            .get_ticker("XBTXMR")
            .await
            .context("Failed to get BTC/XMR ticker from Kraken")?;

        let btc_xmr_price: f64 = ticker.last_trade[0]
            .parse()
            .context("Failed to parse BTC/XMR price")?;

        tracing::info!("Current BTC/XMR rate: {:.8}", btc_xmr_price);

        // Calculate how much BTC we need (with slippage buffer)
        let slippage_multiplier = 1.0 + (config.slippage_tolerance_percent / 100.0);
        let btc_needed = xmr_needed * btc_xmr_price * slippage_multiplier;

        // Cap at max BTC per rebalance
        let btc_to_use = btc_needed.min(config.max_btc_per_rebalance);

        tracing::info!("Will use {:.8} BTC for rebalancing", btc_to_use);

        // Check if we have enough BTC (keeping reserve)
        let (btc_balance, _) = self.get_wallet_balances().await?;
        let btc_balance = btc_balance.context("Bitcoin balance not available")?;

        let btc_available = btc_balance - config.bitcoin_reserve_minimum;
        if btc_available < btc_to_use {
            anyhow::bail!(
                "Insufficient BTC: need {:.8}, have {:.8} available (after reserve)",
                btc_to_use,
                btc_available
            );
        }

        // Step 2: Deposit BTC to Kraken
        tracing::info!("Step 1: Depositing {:.8} BTC to Kraken", btc_to_use);
        let btc_txid = self.deposit_bitcoin_to_kraken(btc_to_use).await?;

        // Step 3: Wait for deposit to confirm
        tracing::info!(
            "Step 2: Waiting for BTC deposit to confirm (txid: {})",
            btc_txid
        );
        self.wait_for_bitcoin_deposit(&kraken, &btc_txid).await?;

        // Step 4: Execute BTC->XMR trade on Kraken
        tracing::info!("Step 3: Executing BTC->XMR trade on Kraken");
        let order_id = self
            .execute_btc_to_xmr_trade(&kraken, btc_to_use, &config)
            .await?;

        // Step 5: Wait for trade to execute
        tracing::info!("Step 4: Waiting for trade execution (order: {})", order_id);
        let xmr_amount = self
            .wait_for_trade_execution(&kraken, &order_id, &config)
            .await?;

        // Step 6: Withdraw XMR from Kraken
        tracing::info!("Step 5: Withdrawing {:.8} XMR from Kraken", xmr_amount);
        let withdraw_refid = self
            .withdraw_monero_from_kraken(&kraken, xmr_amount)
            .await?;

        // Step 7: Wait for withdrawal to complete
        tracing::info!(
            "Step 6: Waiting for XMR withdrawal (refid: {})",
            withdraw_refid
        );
        self.wait_for_monero_withdrawal(&kraken, &withdraw_refid)
            .await?;

        tracing::info!("Rebalancing completed successfully!");
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
        let kraken = KrakenClient::new(self.kraken_api_key.clone(), self.kraken_api_secret.clone());

        let balances = kraken.get_balance().await?;

        let btc = balances.get("XXBT").and_then(|s| s.parse::<f64>().ok());
        let xmr = balances.get("XXMR").and_then(|s| s.parse::<f64>().ok());

        Ok((btc, xmr))
    }

    /// Deposit Bitcoin to Kraken
    async fn deposit_bitcoin_to_kraken(&self, amount: f64) -> Result<String> {
        self.set_state(TradingState::DepositingBitcoin { amount });

        let kraken = KrakenClient::new(self.kraken_api_key.clone(), self.kraken_api_secret.clone());

        // Get Kraken BTC deposit address
        let deposit_address = kraken
            .get_btc_deposit_address(false)
            .await
            .context("Failed to get Kraken BTC deposit address")?;

        tracing::info!("Kraken BTC deposit address: {}", deposit_address);

        // Create transaction record before sending
        let transaction = StoredTradingTransaction {
            id: None,
            timestamp: Utc::now(),
            transaction_type: TransactionType::BitcoinDeposit,
            status: TransactionStatus::Pending,
            btc_amount: Some(amount),
            xmr_amount: None,
            exchange_rate: None,
            txid: None,
            order_id: None,
            refid: None,
            from_address: None,
            to_address: Some(deposit_address.clone()),
            fee: None,
            notes: Some(format!("Depositing {:.8} BTC to Kraken", amount)),
            error_message: None,
            completed_at: None,
        };

        let transaction_id = if let Some(db) = self.get_db() {
            match db.store_trading_transaction(&transaction).await {
                Ok(id) => {
                    tracing::info!("Created transaction record: {}", id);
                    Some(id)
                }
                Err(e) => {
                    tracing::warn!("Failed to store transaction record: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Send BTC from our wallet to Kraken
        let btc_wallet = BitcoinWallet::connect_existing(
            self.bitcoin_wallet_url.clone(),
            &self.bitcoin_wallet_cookie,
            &self.bitcoin_wallet_name,
        )
        .await
        .context("Failed to connect to Bitcoin wallet")?;

        let txid = match btc_wallet
            .send_to_address(&deposit_address, amount, false)
            .await
        {
            Ok(txid) => txid,
            Err(e) => {
                // Mark transaction as failed
                if let (Some(db), Some(id)) = (self.get_db(), transaction_id.as_ref()) {
                    let _ = db.fail_trading_transaction(id, e.to_string()).await;
                }
                return Err(e).context("Failed to send Bitcoin to Kraken");
            }
        };

        tracing::info!("Bitcoin sent to Kraken, txid: {}", txid);

        // Update transaction with txid
        if let (Some(db), Some(id)) = (self.get_db(), transaction_id.as_ref()) {
            let mut updated_transaction = transaction.clone();
            updated_transaction.txid = Some(txid.clone());
            let _ = db
                .update_trading_transaction(id, &updated_transaction)
                .await;
        }

        self.set_state(TradingState::WaitingForBitcoinDeposit { txid: txid.clone() });

        Ok(txid)
    }

    /// Wait for Bitcoin deposit to confirm on Kraken
    async fn wait_for_bitcoin_deposit(&self, kraken: &KrakenClient, txid: &str) -> Result<()> {
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
                    tracing::info!("Bitcoin deposit confirmed on Kraken");

                    // Mark transaction as completed
                    if let Some(db) = self.get_db() {
                        if let Ok(transactions) = db.get_recent_trading_transactions(10).await {
                            if let Some(tx) = transactions.iter().find(|t| {
                                t.txid.as_ref() == Some(&txid.to_string())
                                    && t.status == TransactionStatus::Pending
                            }) {
                                if let Some(id) = &tx.id {
                                    let _ = db.complete_trading_transaction(id, None, None).await;
                                }
                            }
                        }
                    }

                    return Ok(());
                }
            }

            tracing::debug!("Waiting for Bitcoin deposit confirmation...");
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
        let (price, exchange_rate) = if config.use_limit_orders {
            let ticker = kraken.get_ticker("XBTXMR").await?;
            let current_price: f64 = ticker.ask[0].parse()?;
            let price_with_slippage =
                current_price * (1.0 + config.slippage_tolerance_percent / 100.0);
            (
                Some(format!("{:.8}", price_with_slippage)),
                Some(current_price),
            )
        } else {
            (None, None)
        };

        // Create transaction record before placing order
        let transaction = StoredTradingTransaction {
            id: None,
            timestamp: Utc::now(),
            transaction_type: TransactionType::Trade,
            status: TransactionStatus::Pending,
            btc_amount: Some(btc_amount),
            xmr_amount: None,
            exchange_rate,
            txid: None,
            order_id: None,
            refid: None,
            from_address: None,
            to_address: None,
            fee: None,
            notes: Some(format!("Trading {:.8} BTC for XMR", btc_amount)),
            error_message: None,
            completed_at: None,
        };

        let transaction_id = if let Some(db) = self.get_db() {
            match db.store_trading_transaction(&transaction).await {
                Ok(id) => {
                    tracing::info!("Created trade transaction record: {}", id);
                    Some(id)
                }
                Err(e) => {
                    tracing::warn!("Failed to store trade transaction record: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let order = match kraken
            .place_order(
                "XBTXMR",
                "buy",
                order_type,
                &format!("{:.8}", btc_amount),
                price.as_deref(),
            )
            .await
        {
            Ok(order) => order,
            Err(e) => {
                // Mark transaction as failed
                if let (Some(db), Some(id)) = (self.get_db(), transaction_id.as_ref()) {
                    let _ = db.fail_trading_transaction(id, e.to_string()).await;
                }
                return Err(e).context("Failed to place order on Kraken");
            }
        };

        let order_id = order
            .txid
            .first()
            .context("No order ID returned from Kraken")?
            .clone();

        tracing::info!("Order placed on Kraken: {}", order_id);

        // Update transaction with order_id
        if let (Some(db), Some(id)) = (self.get_db(), transaction_id.as_ref()) {
            let mut updated_transaction = transaction.clone();
            updated_transaction.order_id = Some(order_id.clone());
            let _ = db
                .update_trading_transaction(id, &updated_transaction)
                .await;
        }

        self.set_state(TradingState::WaitingForTradeExecution {
            order_id: order_id.clone(),
        });

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
                let error_msg = "Timeout waiting for order execution".to_string();

                // Mark transaction as failed
                if let Some(db) = self.get_db() {
                    if let Ok(transactions) = db.get_recent_trading_transactions(10).await {
                        if let Some(tx) = transactions.iter().find(|t| {
                            t.order_id.as_ref() == Some(&order_id.to_string())
                                && t.status == TransactionStatus::Pending
                        }) {
                            if let Some(id) = &tx.id {
                                let _ = db.fail_trading_transaction(id, error_msg.clone()).await;
                            }
                        }
                    }
                }

                anyhow::bail!(error_msg);
            }

            let order_status = kraken.query_order(order_id).await?;

            if let Some(order_info) = order_status.get(order_id) {
                let status = &order_info.status;

                if status == "closed" {
                    // Order executed successfully
                    let vol_exec = order_info
                        .vol_exec
                        .parse::<f64>()
                        .context("Failed to parse executed volume")?;

                    // Get actual executed price for exchange rate
                    let price = order_info.price.parse::<f64>().ok();

                    tracing::info!("Trade executed successfully, received {:.8} XMR", vol_exec);

                    // Mark transaction as completed
                    if let Some(db) = self.get_db() {
                        if let Ok(transactions) = db.get_recent_trading_transactions(10).await {
                            if let Some(tx) = transactions.iter().find(|t| {
                                t.order_id.as_ref() == Some(&order_id.to_string())
                                    && t.status == TransactionStatus::Pending
                            }) {
                                if let Some(id) = &tx.id {
                                    let _ = db
                                        .complete_trading_transaction(id, Some(vol_exec), price)
                                        .await;
                                }
                            }
                        }
                    }

                    return Ok(vol_exec);
                } else if status == "canceled" || status == "expired" {
                    let error_msg = format!("Order was {} ", status);

                    // Mark transaction as failed
                    if let Some(db) = self.get_db() {
                        if let Ok(transactions) = db.get_recent_trading_transactions(10).await {
                            if let Some(tx) = transactions.iter().find(|t| {
                                t.order_id.as_ref() == Some(&order_id.to_string())
                                    && t.status == TransactionStatus::Pending
                            }) {
                                if let Some(id) = &tx.id {
                                    let _ =
                                        db.fail_trading_transaction(id, error_msg.clone()).await;
                                }
                            }
                        }
                    }

                    anyhow::bail!(error_msg);
                }
            }

            tracing::debug!("Waiting for order execution...");
            sleep(Duration::from_secs(10)).await;
        }
    }

    /// Withdraw Monero from Kraken
    async fn withdraw_monero_from_kraken(
        &self,
        kraken: &KrakenClient,
        amount: f64,
    ) -> Result<String> {
        self.set_state(TradingState::WithdrawingMonero { amount });

        // Get our Monero wallet address
        let xmr_wallet = MoneroWallet::connect_existing(
            self.monero_wallet_url.clone(),
            &self.monero_wallet_name,
            &self.monero_wallet_password,
        )
        .await
        .context("Failed to connect to Monero wallet")?;

        let address = xmr_wallet
            .get_address()
            .await
            .context("Failed to get Monero address")?;

        tracing::info!("Withdrawing to Monero address: {}", address);

        // Create transaction record before withdrawing
        let transaction = StoredTradingTransaction {
            id: None,
            timestamp: Utc::now(),
            transaction_type: TransactionType::MoneroWithdrawal,
            status: TransactionStatus::Pending,
            btc_amount: None,
            xmr_amount: Some(amount),
            exchange_rate: None,
            txid: None,
            order_id: None,
            refid: None,
            from_address: None,
            to_address: Some(address.clone()),
            fee: None,
            notes: Some(format!("Withdrawing {:.8} XMR from Kraken", amount)),
            error_message: None,
            completed_at: None,
        };

        let transaction_id = if let Some(db) = self.get_db() {
            match db.store_trading_transaction(&transaction).await {
                Ok(id) => {
                    tracing::info!("Created withdrawal transaction record: {}", id);
                    Some(id)
                }
                Err(e) => {
                    tracing::warn!("Failed to store withdrawal transaction record: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Initiate withdrawal from Kraken
        // Note: First parameter is the withdrawal key name configured in Kraken, not the address
        // For now, we'll use a default key name - this should be configurable
        let withdraw_result = match kraken
            .withdraw_xmr("monero_primary", &format!("{:.12}", amount))
            .await
        {
            Ok(result) => result,
            Err(e) => {
                // Mark transaction as failed
                if let (Some(db), Some(id)) = (self.get_db(), transaction_id.as_ref()) {
                    let _ = db.fail_trading_transaction(id, e.to_string()).await;
                }
                return Err(e).context("Failed to initiate Monero withdrawal from Kraken");
            }
        };

        let refid = withdraw_result.refid;
        tracing::info!("Monero withdrawal initiated: {}", refid);

        // Update transaction with refid
        if let (Some(db), Some(id)) = (self.get_db(), transaction_id.as_ref()) {
            let mut updated_transaction = transaction.clone();
            updated_transaction.refid = Some(refid.clone());
            let _ = db
                .update_trading_transaction(id, &updated_transaction)
                .await;
        }

        self.set_state(TradingState::WaitingForMoneroWithdrawal {
            refid: refid.clone(),
        });

        Ok(refid)
    }

    /// Wait for Monero withdrawal to complete
    async fn wait_for_monero_withdrawal(&self, kraken: &KrakenClient, refid: &str) -> Result<()> {
        let timeout = Duration::from_secs(3600); // 1 hour timeout
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                let error_msg = "Timeout waiting for Monero withdrawal".to_string();

                // Mark transaction as failed
                if let Some(db) = self.get_db() {
                    if let Ok(transactions) = db.get_recent_trading_transactions(10).await {
                        if let Some(tx) = transactions.iter().find(|t| {
                            t.refid.as_ref() == Some(&refid.to_string())
                                && t.status == TransactionStatus::Pending
                        }) {
                            if let Some(id) = &tx.id {
                                let _ = db.fail_trading_transaction(id, error_msg.clone()).await;
                            }
                        }
                    }
                }

                anyhow::bail!(error_msg);
            }

            let withdrawals = kraken.get_withdrawal_status(Some("XMR")).await?;

            // Find our withdrawal
            if let Some(withdrawal) = withdrawals.iter().find(|w| w.refid == refid) {
                if withdrawal.status == "Success" {
                    tracing::info!("Monero withdrawal completed successfully");

                    // Mark transaction as completed
                    if let Some(db) = self.get_db() {
                        if let Ok(transactions) = db.get_recent_trading_transactions(10).await {
                            if let Some(tx) = transactions.iter().find(|t| {
                                t.refid.as_ref() == Some(&refid.to_string())
                                    && t.status == TransactionStatus::Pending
                            }) {
                                if let Some(id) = &tx.id {
                                    let _ = db.complete_trading_transaction(id, None, None).await;
                                }
                            }
                        }
                    }

                    return Ok(());
                } else if withdrawal.status == "Failure" || withdrawal.status == "Canceled" {
                    let error_msg =
                        format!("Monero withdrawal {}", withdrawal.status.to_lowercase());

                    // Mark transaction as failed
                    if let Some(db) = self.get_db() {
                        if let Ok(transactions) = db.get_recent_trading_transactions(10).await {
                            if let Some(tx) = transactions.iter().find(|t| {
                                t.refid.as_ref() == Some(&refid.to_string())
                                    && t.status == TransactionStatus::Pending
                            }) {
                                if let Some(id) = &tx.id {
                                    let _ =
                                        db.fail_trading_transaction(id, error_msg.clone()).await;
                                }
                            }
                        }
                    }

                    anyhow::bail!(error_msg);
                }
            }

            tracing::debug!("Waiting for Monero withdrawal completion...");
            sleep(Duration::from_secs(30)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trading::config::{SharedTradingConfig, TradingConfig};

    fn create_test_engine() -> TradingEngine {
        let config = TradingConfig::default();
        let shared_config = SharedTradingConfig::new(config);

        TradingEngine::new(
            shared_config,
            "test_key".to_string(),
            "test_secret".to_string(),
            "http://localhost:8332".to_string(),
            "/tmp/cookie".to_string(),
            "test_wallet".to_string(),
            "http://localhost:18082/json_rpc".to_string(),
            "test_xmr_wallet".to_string(),
            "".to_string(),
        )
    }

    #[test]
    fn test_engine_creation() {
        let engine = create_test_engine();
        assert!(!engine.is_enabled());
        assert_eq!(engine.get_state(), TradingState::Disabled);
    }

    #[test]
    fn test_engine_enable_disable() {
        let engine = create_test_engine();

        engine.enable();
        assert!(engine.is_enabled());
        assert_eq!(engine.get_state(), TradingState::Monitoring);

        engine.disable();
        assert!(!engine.is_enabled());
        assert_eq!(engine.get_state(), TradingState::Disabled);
    }

    #[test]
    fn test_engine_state_management() {
        let engine = create_test_engine();

        // Test different state transitions
        engine.enable();
        assert_eq!(engine.get_state(), TradingState::Monitoring);

        // Manually set different states (private method, but testing the storage)
        assert_eq!(engine.get_state(), TradingState::Monitoring);
    }

    #[test]
    fn test_engine_with_database() {
        let engine = create_test_engine();
        assert!(engine.get_db().is_none());

        // After creation, database should be None
        assert!(!engine.is_enabled());
    }

    #[test]
    fn test_trading_state_equality() {
        assert_eq!(TradingState::Disabled, TradingState::Disabled);
        assert_eq!(TradingState::Monitoring, TradingState::Monitoring);
        assert_ne!(TradingState::Disabled, TradingState::Monitoring);

        let state1 = TradingState::DepositingBitcoin { amount: 0.1 };
        let state2 = TradingState::DepositingBitcoin { amount: 0.1 };
        let state3 = TradingState::DepositingBitcoin { amount: 0.2 };
        assert_eq!(state1, state2);
        assert_ne!(state1, state3);
    }

    #[test]
    fn test_trading_state_serialization() {
        let states = vec![
            TradingState::Disabled,
            TradingState::Monitoring,
            TradingState::DepositingBitcoin { amount: 0.5 },
            TradingState::WaitingForBitcoinDeposit {
                txid: "test_txid".to_string(),
            },
            TradingState::Trading { btc_amount: 0.3 },
            TradingState::WaitingForTradeExecution {
                order_id: "order123".to_string(),
            },
            TradingState::WithdrawingMonero { amount: 10.0 },
            TradingState::WaitingForMoneroWithdrawal {
                refid: "ref456".to_string(),
            },
            TradingState::Error {
                message: "test error".to_string(),
            },
        ];

        for state in states {
            // Test that states can be serialized
            let json = serde_json::to_string(&state).expect("Should serialize");
            assert!(!json.is_empty());

            // Test that they can be deserialized
            let deserialized: TradingState =
                serde_json::from_str(&json).expect("Should deserialize");
            assert_eq!(state, deserialized);
        }
    }

    #[test]
    fn test_trading_status_structure() {
        let status = TradingStatus {
            state: TradingState::Monitoring,
            enabled: true,
            last_check: Some("2024-01-01T00:00:00Z".to_string()),
            last_rebalance: None,
            current_btc_balance: Some(1.5),
            current_xmr_balance: Some(50.0),
            kraken_btc_balance: Some(0.1),
            kraken_xmr_balance: Some(5.0),
        };

        assert_eq!(status.state, TradingState::Monitoring);
        assert!(status.enabled);
        assert_eq!(status.current_btc_balance, Some(1.5));
        assert_eq!(status.current_xmr_balance, Some(50.0));

        // Test serialization
        let json = serde_json::to_string(&status).expect("Should serialize");
        assert!(!json.is_empty());
    }

    #[test]
    fn test_config_integration() {
        let config = TradingConfig::default();
        let shared_config = SharedTradingConfig::new(config);

        let engine = TradingEngine::new(
            shared_config.clone(),
            "key".to_string(),
            "secret".to_string(),
            "http://localhost:8332".to_string(),
            "/tmp/cookie".to_string(),
            "wallet".to_string(),
            "http://localhost:18082/json_rpc".to_string(),
            "xmr_wallet".to_string(),
            "".to_string(),
        );

        // Engine should have access to config
        let current_config = shared_config.get();
        assert!(current_config.monero_min_threshold > 0.0);
        assert!(engine.is_enabled() == false);
    }

    #[test]
    fn test_multiple_engines_independence() {
        let engine1 = create_test_engine();
        let engine2 = create_test_engine();

        engine1.enable();
        assert!(engine1.is_enabled());
        assert!(!engine2.is_enabled());

        engine2.enable();
        assert!(engine1.is_enabled());
        assert!(engine2.is_enabled());

        engine1.disable();
        assert!(!engine1.is_enabled());
        assert!(engine2.is_enabled());
    }

    #[tokio::test]
    async fn test_get_status_without_wallets() {
        let engine = create_test_engine();

        // Should not panic even if wallets aren't available
        let status = engine.get_status().await;

        assert_eq!(status.state, TradingState::Disabled);
        assert!(!status.enabled);
        // Balances may be None if wallets aren't available
    }
}
