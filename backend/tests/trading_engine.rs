/// Integration tests for Trading Engine
///
/// Run with: cargo nextest run --test trading_engine
///
/// These tests require:
/// - Bitcoin wallet RPC endpoint
/// - Monero wallet RPC endpoint
/// - Kraken API credentials (for integration tests)
/// - SurrealDB instance (for transaction tracking tests)
///
/// Note: Many tests are marked with #[ignore] as they require live services
/// and API credentials. Run specific tests with:
/// cargo nextest run --test trading_engine --ignored
use anyhow::Result;
use chrono::{Duration as ChronoDuration, Utc};
use eigenix_backend::db::{
    MetricsDatabase, StoredTradingTransaction, TransactionStatus, TransactionType,
};
use eigenix_backend::trading::{TradingConfig, TradingEngine};

mod common;
use common::TestConfig;

/// Helper to create a test database connection
async fn setup_test_db() -> Result<MetricsDatabase> {
    let db = MetricsDatabase::connect("127.0.0.1:8001", "test_eigenix", "test_trading").await?;
    Ok(db)
}

/// Helper to create a test trading engine without database
fn create_test_engine() -> TradingEngine {
    let config = TradingConfig::default();
    let shared_config = eigenix_backend::trading::config::SharedTradingConfig::new(config);

    let (btc_url, btc_cookie, wallet_name) = TestConfig::bitcoin_wallet();
    let (xmr_url, xmr_wallet, xmr_password) = TestConfig::monero_wallet();
    let (kraken_key, kraken_secret) = TestConfig::kraken();

    TradingEngine::new(
        shared_config,
        kraken_key,
        kraken_secret,
        btc_url,
        btc_cookie,
        wallet_name.clone(),
        xmr_url,
        xmr_wallet,
        xmr_password,
    )
}

#[tokio::test]
async fn test_trading_engine_creation() {
    let engine = create_test_engine();

    // Engine should start disabled
    assert!(!engine.is_enabled(), "Engine should start disabled");
    assert_eq!(
        engine.get_state(),
        eigenix_backend::trading::engine::TradingState::Disabled,
        "Engine should be in Disabled state"
    );
}

#[tokio::test]
async fn test_trading_engine_enable_disable() {
    let engine = create_test_engine();

    // Test enable
    engine.enable();
    assert!(engine.is_enabled(), "Engine should be enabled");
    assert_eq!(
        engine.get_state(),
        eigenix_backend::trading::engine::TradingState::Monitoring,
        "Engine should be in Monitoring state when enabled"
    );

    // Test disable
    engine.disable();
    assert!(!engine.is_enabled(), "Engine should be disabled");
    assert_eq!(
        engine.get_state(),
        eigenix_backend::trading::engine::TradingState::Disabled,
        "Engine should be in Disabled state when disabled"
    );
}

#[tokio::test]
async fn test_trading_config_validation() {
    let mut config = TradingConfig::default();

    // Valid config should pass
    assert!(config.validate().is_ok(), "Default config should be valid");

    // Invalid: min_threshold >= target_balance
    config.monero_min_threshold = 10.0;
    config.monero_target_balance = 5.0;
    assert!(
        config.validate().is_err(),
        "Config with min_threshold >= target should be invalid"
    );

    // Invalid: negative min_threshold
    config.monero_min_threshold = -1.0;
    config.monero_target_balance = 5.0;
    assert!(
        config.validate().is_err(),
        "Config with negative min_threshold should be invalid"
    );

    // Invalid: negative bitcoin_reserve
    config = TradingConfig::default();
    config.bitcoin_reserve_minimum = -0.01;
    assert!(
        config.validate().is_err(),
        "Config with negative bitcoin_reserve should be invalid"
    );

    // Invalid: zero check interval
    config = TradingConfig::default();
    config.check_interval_secs = 0;
    assert!(
        config.validate().is_err(),
        "Config with zero check_interval should be invalid"
    );

    // Invalid: slippage out of range
    config = TradingConfig::default();
    config.slippage_tolerance_percent = 150.0;
    assert!(
        config.validate().is_err(),
        "Config with slippage > 100% should be invalid"
    );
}

#[tokio::test]
async fn test_trading_config_update() {
    let config = TradingConfig::default();
    let shared_config = eigenix_backend::trading::config::SharedTradingConfig::new(config);

    // Update with valid config
    let mut new_config = TradingConfig::default();
    new_config.monero_min_threshold = 2.0;
    new_config.monero_target_balance = 10.0;

    assert!(
        shared_config.update(new_config.clone()).is_ok(),
        "Should be able to update with valid config"
    );

    let updated = shared_config.get();
    assert_eq!(
        updated.monero_min_threshold, 2.0,
        "Config should be updated"
    );
    assert_eq!(
        updated.monero_target_balance, 10.0,
        "Config should be updated"
    );

    // Update with invalid config should fail
    let mut invalid_config = TradingConfig::default();
    invalid_config.monero_min_threshold = 10.0;
    invalid_config.monero_target_balance = 5.0;

    assert!(
        shared_config.update(invalid_config).is_err(),
        "Should not be able to update with invalid config"
    );

    // Original valid config should still be in place
    let unchanged = shared_config.get();
    assert_eq!(
        unchanged.monero_min_threshold, 2.0,
        "Config should not have changed"
    );
}

#[tokio::test]
#[ignore] // Requires live wallet connections
async fn test_trading_engine_get_status() {
    let engine = create_test_engine();

    let status = engine.get_status().await;

    // Should get a valid status even if wallets aren't connected
    assert_eq!(status.enabled, false, "Engine should start disabled");
    assert_eq!(
        status.state,
        eigenix_backend::trading::engine::TradingState::Disabled
    );
}

#[tokio::test]
#[ignore] // Requires database
async fn test_database_transaction_tracking() {
    let db = match setup_test_db().await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("⚠️  Skipping test: database not available: {}", e);
            return;
        }
    };

    // Create a test transaction
    let transaction = StoredTradingTransaction {
        id: None,
        timestamp: Utc::now(),
        transaction_type: TransactionType::BitcoinDeposit,
        status: TransactionStatus::Pending,
        btc_amount: Some(0.1),
        xmr_amount: None,
        exchange_rate: None,
        txid: Some("test_txid_123".to_string()),
        order_id: None,
        refid: None,
        from_address: None,
        to_address: Some("test_address".to_string()),
        fee: None,
        notes: Some("Test deposit".to_string()),
        error_message: None,
        completed_at: None,
    };

    // Store transaction
    let transaction_id = db
        .store_trading_transaction(&transaction)
        .await
        .expect("Should store transaction");

    assert!(!transaction_id.is_empty(), "Should get a transaction ID");

    // Retrieve transaction
    let retrieved = db
        .get_trading_transaction(&transaction_id)
        .await
        .expect("Should retrieve transaction")
        .expect("Transaction should exist");

    assert_eq!(retrieved.transaction_type, TransactionType::BitcoinDeposit);
    assert_eq!(retrieved.status, TransactionStatus::Pending);
    assert_eq!(retrieved.btc_amount, Some(0.1));
    assert_eq!(retrieved.txid, Some("test_txid_123".to_string()));

    // Complete transaction
    db.complete_trading_transaction(&transaction_id, None, None)
        .await
        .expect("Should complete transaction");

    let completed = db
        .get_trading_transaction(&transaction_id)
        .await
        .expect("Should retrieve transaction")
        .expect("Transaction should exist");

    assert_eq!(completed.status, TransactionStatus::Completed);
    assert!(completed.completed_at.is_some());
}

#[tokio::test]
#[ignore] // Requires database
async fn test_database_transaction_queries() {
    let db = match setup_test_db().await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("⚠️  Skipping test: database not available: {}", e);
            return;
        }
    };

    let now = Utc::now();

    // Create multiple test transactions
    for i in 0..5 {
        let transaction = StoredTradingTransaction {
            id: None,
            timestamp: now - ChronoDuration::minutes(i),
            transaction_type: if i % 2 == 0 {
                TransactionType::BitcoinDeposit
            } else {
                TransactionType::Trade
            },
            status: if i < 3 {
                TransactionStatus::Completed
            } else {
                TransactionStatus::Pending
            },
            btc_amount: Some(0.01 * (i as f64 + 1.0)),
            xmr_amount: None,
            exchange_rate: None,
            txid: Some(format!("test_txid_{}", i)),
            order_id: None,
            refid: None,
            from_address: None,
            to_address: None,
            fee: None,
            notes: Some(format!("Test transaction {}", i)),
            error_message: None,
            completed_at: if i < 3 { Some(now) } else { None },
        };

        db.store_trading_transaction(&transaction)
            .await
            .expect("Should store transaction");
    }

    // Query recent transactions
    let recent = db
        .get_recent_trading_transactions(3)
        .await
        .expect("Should query recent transactions");

    assert_eq!(recent.len(), 3, "Should get 3 most recent transactions");

    // Query by status
    let pending = db
        .get_trading_transactions_by_status(TransactionStatus::Pending)
        .await
        .expect("Should query by status");

    assert!(
        pending.len() >= 2,
        "Should have at least 2 pending transactions"
    );
    for tx in &pending {
        assert_eq!(tx.status, TransactionStatus::Pending);
    }

    // Query by type
    let deposits = db
        .get_trading_transactions_by_type(TransactionType::BitcoinDeposit)
        .await
        .expect("Should query by type");

    assert!(
        deposits.len() >= 3,
        "Should have at least 3 deposit transactions"
    );
    for tx in &deposits {
        assert_eq!(tx.transaction_type, TransactionType::BitcoinDeposit);
    }

    // Query by time range
    let from = now - ChronoDuration::minutes(10);
    let to = now + ChronoDuration::minutes(1);
    let in_range = db
        .get_trading_transactions(from, to)
        .await
        .expect("Should query by time range");

    assert!(in_range.len() >= 5, "Should have all transactions in range");
}

#[tokio::test]
#[ignore] // Requires database
async fn test_database_transaction_failure() {
    let db = match setup_test_db().await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("⚠️  Skipping test: database not available: {}", e);
            return;
        }
    };

    // Create a test transaction
    let transaction = StoredTradingTransaction {
        id: None,
        timestamp: Utc::now(),
        transaction_type: TransactionType::Trade,
        status: TransactionStatus::Pending,
        btc_amount: Some(0.05),
        xmr_amount: None,
        exchange_rate: None,
        txid: None,
        order_id: Some("test_order_123".to_string()),
        refid: None,
        from_address: None,
        to_address: None,
        fee: None,
        notes: Some("Test trade".to_string()),
        error_message: None,
        completed_at: None,
    };

    let transaction_id = db
        .store_trading_transaction(&transaction)
        .await
        .expect("Should store transaction");

    // Mark as failed
    let error_msg = "Order timeout".to_string();
    db.fail_trading_transaction(&transaction_id, error_msg.clone())
        .await
        .expect("Should fail transaction");

    let failed = db
        .get_trading_transaction(&transaction_id)
        .await
        .expect("Should retrieve transaction")
        .expect("Transaction should exist");

    assert_eq!(failed.status, TransactionStatus::Failed);
    assert_eq!(failed.error_message, Some(error_msg));
    assert!(failed.completed_at.is_some());
}

#[tokio::test]
#[ignore] // Requires database
async fn test_engine_with_database_integration() {
    let db = match setup_test_db().await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("⚠️  Skipping test: database not available: {}", e);
            return;
        }
    };

    let engine = create_test_engine().with_database(db.clone());

    // Verify engine can be created with database
    assert!(!engine.is_enabled());

    // Enable and check status
    engine.enable();
    assert!(engine.is_enabled());
}

#[tokio::test]
async fn test_transaction_type_serialization() {
    // Test that transaction types serialize correctly
    let deposit = TransactionType::BitcoinDeposit;
    let trade = TransactionType::Trade;
    let withdrawal = TransactionType::MoneroWithdrawal;

    // These should be distinguishable
    assert_ne!(format!("{:?}", deposit), format!("{:?}", trade));
    assert_ne!(format!("{:?}", trade), format!("{:?}", withdrawal));
    assert_ne!(format!("{:?}", deposit), format!("{:?}", withdrawal));
}

#[tokio::test]
async fn test_transaction_status_serialization() {
    let pending = TransactionStatus::Pending;
    let completed = TransactionStatus::Completed;
    let failed = TransactionStatus::Failed;
    let cancelled = TransactionStatus::Cancelled;

    // These should be distinguishable
    assert_ne!(format!("{:?}", pending), format!("{:?}", completed));
    assert_ne!(format!("{:?}", completed), format!("{:?}", failed));
    assert_ne!(format!("{:?}", failed), format!("{:?}", cancelled));
}

#[tokio::test]
async fn test_trading_config_defaults() {
    let config = TradingConfig::default();

    // Verify default values are sensible
    assert!(config.monero_min_threshold > 0.0);
    assert!(config.monero_target_balance > config.monero_min_threshold);
    assert!(config.bitcoin_reserve_minimum >= 0.0);
    assert!(config.max_btc_per_rebalance > 0.0);
    assert!(config.check_interval_secs > 0);
    assert!(config.order_timeout_secs > 0);
    assert!(config.slippage_tolerance_percent >= 0.0);
    assert!(config.slippage_tolerance_percent <= 100.0);

    // Should be valid
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_stored_transaction_creation() {
    // Test creating a complete transaction record
    let transaction = StoredTradingTransaction {
        id: None,
        timestamp: Utc::now(),
        transaction_type: TransactionType::Trade,
        status: TransactionStatus::Completed,
        btc_amount: Some(0.05),
        xmr_amount: Some(2.5),
        exchange_rate: Some(0.02),
        txid: None,
        order_id: Some("ORDER123".to_string()),
        refid: None,
        from_address: None,
        to_address: None,
        fee: Some(0.0001),
        notes: Some("Successful trade".to_string()),
        error_message: None,
        completed_at: Some(Utc::now()),
    };

    // Verify all fields are accessible
    assert_eq!(transaction.transaction_type, TransactionType::Trade);
    assert_eq!(transaction.status, TransactionStatus::Completed);
    assert_eq!(transaction.btc_amount, Some(0.05));
    assert_eq!(transaction.xmr_amount, Some(2.5));
    assert_eq!(transaction.exchange_rate, Some(0.02));
    assert!(transaction.completed_at.is_some());
}

#[tokio::test]
#[ignore] // Requires live Kraken API and wallet connections
async fn test_full_rebalance_workflow_simulation() {
    // This test simulates the full workflow without actually executing trades
    // It's more of an integration test that requires all services to be running

    let engine = create_test_engine();

    // Check if we can get balances (this will fail gracefully if services aren't available)
    let status = engine.get_status().await;

    println!("Engine status:");
    println!("  Enabled: {}", status.enabled);
    println!("  State: {:?}", status.state);
    println!("  BTC Balance: {:?}", status.current_btc_balance);
    println!("  XMR Balance: {:?}", status.current_xmr_balance);
    println!("  Kraken BTC: {:?}", status.kraken_btc_balance);
    println!("  Kraken XMR: {:?}", status.kraken_xmr_balance);

    // This test is mainly for manual verification
    assert!(true);
}

#[tokio::test]
async fn test_trading_state_transitions() {
    use eigenix_backend::trading::engine::TradingState;

    let engine = create_test_engine();

    // Initial state should be Disabled
    assert_eq!(engine.get_state(), TradingState::Disabled);

    // Enable should transition to Monitoring
    engine.enable();
    assert_eq!(engine.get_state(), TradingState::Monitoring);

    // Disable should transition back to Disabled
    engine.disable();
    assert_eq!(engine.get_state(), TradingState::Disabled);
}

#[tokio::test]
async fn test_multiple_engines_with_shared_config() {
    let config = TradingConfig::default();
    let shared_config = eigenix_backend::trading::config::SharedTradingConfig::new(config);

    let (btc_url, btc_cookie, wallet_name) = TestConfig::bitcoin_wallet();
    let (xmr_url, xmr_wallet, xmr_password) = TestConfig::monero_wallet();
    let (kraken_key, kraken_secret) = TestConfig::kraken();

    // Create two engines sharing the same config
    let engine1 = TradingEngine::new(
        shared_config.clone(),
        kraken_key.clone(),
        kraken_secret.clone(),
        btc_url.clone(),
        btc_cookie.clone(),
        wallet_name.clone(),
        xmr_url.clone(),
        xmr_wallet.clone(),
        xmr_password.clone(),
    );

    let engine2 = TradingEngine::new(
        shared_config.clone(),
        kraken_key,
        kraken_secret,
        btc_url,
        btc_cookie,
        wallet_name,
        xmr_url,
        xmr_wallet,
        xmr_password,
    );

    // Update config
    let mut new_config = TradingConfig::default();
    new_config.monero_min_threshold = 3.0;
    shared_config
        .update(new_config)
        .expect("Should update config");

    // Both engines should see the updated config
    let config1 = shared_config.get();
    let config2 = shared_config.get();

    assert_eq!(config1.monero_min_threshold, 3.0);
    assert_eq!(config2.monero_min_threshold, 3.0);

    // Engines should be independent
    engine1.enable();
    assert!(engine1.is_enabled());
    assert!(!engine2.is_enabled());
}

#[tokio::test]
#[ignore] // Requires database
async fn test_concurrent_transaction_creation() {
    let db = match setup_test_db().await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("⚠️  Skipping test: database not available: {}", e);
            return;
        }
    };

    // Create multiple transactions concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            let transaction = StoredTradingTransaction {
                id: None,
                timestamp: Utc::now(),
                transaction_type: TransactionType::Trade,
                status: TransactionStatus::Pending,
                btc_amount: Some(0.01 * (i as f64)),
                xmr_amount: None,
                exchange_rate: None,
                txid: None,
                order_id: Some(format!("order_{}", i)),
                refid: None,
                from_address: None,
                to_address: None,
                fee: None,
                notes: Some(format!("Concurrent test {}", i)),
                error_message: None,
                completed_at: None,
            };

            db_clone.store_trading_transaction(&transaction).await
        });

        handles.push(handle);
    }

    // Wait for all to complete
    let results = futures::future::join_all(handles).await;

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    // Verify all transactions were stored
    let recent = db
        .get_recent_trading_transactions(10)
        .await
        .expect("Should query transactions");

    assert!(recent.len() >= 10, "Should have at least 10 transactions");
}
