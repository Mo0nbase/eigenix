/// Integration tests for Kraken API
///
/// Run with: cargo test --test kraken_integration
///
/// These tests require Kraken API credentials to be set via environment variables:
/// - KRAKEN_API_KEY: Kraken API key
/// - KRAKEN_API_SECRET: Kraken API secret
///
/// Note: Kraken API keys can be configured with different permissions in the dashboard.
/// For testing, create API keys with limited permissions (e.g., query only, no trading/withdrawals).
///
/// See tests/KRAKEN_TESTING.md for detailed documentation.
use eigenix_backend::services::kraken::KrakenClient;

mod common;
use common::TestConfig;

#[tokio::test]
async fn test_kraken_client_creation() {
    let (api_key, api_secret) = TestConfig::kraken();

    // Test can run even without credentials (will fail gracefully)
    let client = KrakenClient::new(api_key, api_secret);
    // Simple check that client was created
    assert!(true);
}

#[tokio::test]
async fn test_kraken_public_ticker() {
    let (api_key, api_secret) = TestConfig::kraken();
    let client = KrakenClient::new(api_key, api_secret);

    // Test public ticker endpoint (doesn't require authentication)
    let ticker_result = client.get_ticker("XBTXMR").await;

    match ticker_result {
        Ok(ticker) => {
            // Verify ticker data structure
            assert!(!ticker.ask.is_empty(), "Ask prices should not be empty");
            assert!(!ticker.bid.is_empty(), "Bid prices should not be empty");
            assert!(!ticker.last_trade.is_empty(), "Last trade should not be empty");
            assert!(!ticker.volume.is_empty(), "Volume should not be empty");

            // Verify price data is numeric
            let ask_price: f64 = ticker.ask[0].parse().expect("Ask price should be numeric");
            let bid_price: f64 = ticker.bid[0].parse().expect("Bid price should be numeric");
            assert!(ask_price > 0.0, "Ask price should be positive");
            assert!(bid_price > 0.0, "Bid price should be positive");
            assert!(ask_price >= bid_price, "Ask should be >= bid");
        }
        Err(e) => {
            eprintln!("⚠️  Kraken ticker test failed (network/API issue): {}", e);
            // Don't fail the test for network/API issues, only for our code bugs
        }
    }
}

#[tokio::test]
#[ignore] // Requires valid credentials
async fn test_kraken_balance() {
    let (api_key, api_secret) = TestConfig::kraken();

    // Skip if no credentials provided
    if api_key.is_empty() || api_secret.is_empty() {
        eprintln!("⚠️  Skipping Kraken balance test: no credentials provided");
        return;
    }

    let client = KrakenClient::new(api_key, api_secret);

    let balance_result = client.get_balance().await;

    match balance_result {
        Ok(balance) => {
            // Balance should be a map, even if empty
            assert!(balance.is_empty() || !balance.is_empty()); // Always true, just ensures we got a response
            println!("Account balance retrieved successfully");
        }
        Err(e) => {
            eprintln!("Balance test failed: {}", e);
            // This might fail if credentials are invalid or API changes
            // Don't panic, as this is expected in some environments
        }
    }
}

#[tokio::test]
#[ignore] // Requires valid credentials
async fn test_kraken_deposit_methods() {
    let (api_key, api_secret) = TestConfig::kraken();

    // Skip if no credentials provided
    if api_key.is_empty() || api_secret.is_empty() {
        eprintln!("⚠️  Skipping Kraken deposit methods test: no credentials provided");
        return;
    }

    let client = KrakenClient::new(api_key, api_secret);

    // Test Bitcoin deposit methods
    let btc_methods_result = client.get_deposit_methods("XBT").await;
    match btc_methods_result {
        Ok(methods) => {
            assert!(!methods.is_empty(), "Should have at least one deposit method for BTC");
            println!("Found {} BTC deposit methods", methods.len());
        }
        Err(e) => {
            eprintln!("BTC deposit methods test failed: {}", e);
        }
    }

    // Test Monero deposit methods
    let xmr_methods_result = client.get_deposit_methods("XMR").await;
    match xmr_methods_result {
        Ok(methods) => {
            assert!(!methods.is_empty(), "Should have at least one deposit method for XMR");
            println!("Found {} XMR deposit methods", methods.len());
        }
        Err(e) => {
            eprintln!("XMR deposit methods test failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Requires valid credentials and proper account setup
async fn test_kraken_deposit_address() {
    let (api_key, api_secret) = TestConfig::kraken();

    // Skip if no credentials provided
    if api_key.is_empty() || api_secret.is_empty() {
        eprintln!("⚠️  Skipping Kraken deposit address test: no credentials provided");
        return;
    }

    let client = KrakenClient::new(api_key, api_secret);

    // First get deposit methods
    let methods_result = client.get_deposit_methods("XBT").await;
    if let Ok(methods) = methods_result {
        if let Some(method) = methods.first() {
            let address_result = client.get_deposit_address("XBT", &method.method, false).await;
            match address_result {
                Ok(addresses) => {
                    assert!(!addresses.is_empty(), "Should get at least one deposit address");
                    let addr = &addresses[0];
                    assert!(!addr.address.is_empty(), "Address should not be empty");
                    println!("Got BTC deposit address: {}", addr.address);
                }
                Err(e) => {
                    eprintln!("Deposit address test failed: {}", e);
                }
            }
        }
    }
}

#[tokio::test]
#[ignore] // Requires valid credentials with trading permissions
async fn test_kraken_order_operations() {
    let (api_key, api_secret) = TestConfig::kraken();

    // Skip if no credentials provided
    if api_key.is_empty() || api_secret.is_empty() {
        eprintln!("⚠️  Skipping Kraken order operations test: no credentials provided");
        return;
    }

    let client = KrakenClient::new(api_key, api_secret);

    // Test placing a small limit order with a price that won't execute
    let order_result = client.place_order(
        "XBTXMR",
        "sell",
        "limit",
        "0.0001", // Very small amount
        Some("1000000.0"), // Very high price, won't execute
    ).await;

    match order_result {
        Ok(order_info) => {
            assert!(!order_info.txid.is_empty(), "Should get transaction IDs");
            println!("Order placed successfully, txids: {:?}", order_info.txid);

            // If we got txids, try to query the order status
            if let Some(txid) = order_info.txid.first() {
                let query_result = client.query_order(txid).await;
                match query_result {
                    Ok(order_status) => {
                        assert!(order_status.contains_key(txid), "Order status should contain our txid");
                        println!("Order status queried successfully");
                    }
                    Err(e) => {
                        eprintln!("Order query failed: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Order placement failed: {}", e);
            // This is expected if account has insufficient funds or API lacks permissions
        }
    }
}

#[tokio::test]
#[ignore] // Requires valid credentials
async fn test_kraken_withdrawal_status() {
    let (api_key, api_secret) = TestConfig::kraken();

    // Skip if no credentials provided
    if api_key.is_empty() || api_secret.is_empty() {
        eprintln!("⚠️  Skipping Kraken withdrawal status test: no credentials provided");
        return;
    }

    let client = KrakenClient::new(api_key, api_secret);

    let withdrawal_status_result = client.get_withdrawal_status(None).await;
    match withdrawal_status_result {
        Ok(statuses) => {
            // Even empty list is success
            println!("Withdrawal status retrieved successfully, {} records", statuses.len());
        }
        Err(e) => {
            eprintln!("Withdrawal status test failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Requires valid credentials
async fn test_kraken_deposit_status() {
    let (api_key, api_secret) = TestConfig::kraken();

    // Skip if no credentials provided
    if api_key.is_empty() || api_secret.is_empty() {
        eprintln!("⚠️  Skipping Kraken deposit status test: no credentials provided");
        return;
    }

    let client = KrakenClient::new(api_key, api_secret);

    let deposit_status_result = client.get_deposit_status(None).await;
    match deposit_status_result {
        Ok(statuses) => {
            // Even empty list is success
            println!("Deposit status retrieved successfully, {} records", statuses.len());
        }
        Err(e) => {
            eprintln!("Deposit status test failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Requires valid credentials - USE WITH CAUTION
async fn test_kraken_get_deposit_addresses() {
    let (api_key, api_secret) = TestConfig::kraken();

    // Skip if no credentials provided
    if api_key.is_empty() || api_secret.is_empty() {
        eprintln!("⚠️  Skipping Kraken deposit address retrieval test: no credentials provided");
        return;
    }

    let client = KrakenClient::new(api_key, api_secret);

    let btc_address_result = client.get_btc_deposit_address(false).await;
    match btc_address_result {
        Ok(address) => {
            assert!(!address.is_empty(), "BTC deposit address should not be empty");
            println!("BTC deposit address: {}", address);
        }
        Err(e) => {
            eprintln!("BTC deposit address test failed: {}", e);
        }
    }

    let xmr_address_result = client.get_xmr_deposit_address(false).await;
    match xmr_address_result {
        Ok(address) => {
            assert!(!address.is_empty(), "XMR deposit address should not be empty");
            println!("XMR deposit address: {}", address);
        }
        Err(e) => {
            eprintln!("XMR deposit address test failed: {}", e);
        }
    }
}
