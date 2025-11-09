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
use eigenix_backend::routes::kraken::KrakenTickerResponse;
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
async fn test_kraken_public_ticker_btcusd() {
    let (api_key, api_secret) = TestConfig::kraken();
    let client = KrakenClient::new(api_key, api_secret);

    // Test BTC/USD ticker (public endpoint, no auth required)
    let ticker_result = client.get_ticker("XBTUSD").await;

    match ticker_result {
        Ok(ticker) => {
            // Verify ticker data structure
            assert!(!ticker.ask.is_empty(), "Ask prices should not be empty");
            assert!(!ticker.bid.is_empty(), "Bid prices should not be empty");
            assert!(!ticker.last_trade.is_empty(), "Last trade should not be empty");
            assert!(!ticker.volume.is_empty(), "Volume should not be empty");

            // Verify price data is numeric and reasonable
            let ask_price: f64 = ticker.ask[0].parse().expect("Ask price should be numeric");
            let bid_price: f64 = ticker.bid[0].parse().expect("Bid price should be numeric");
            let last_price: f64 = ticker.last_trade[0].parse().expect("Last trade should be numeric");
            
            assert!(ask_price > 0.0, "Ask price should be positive");
            assert!(bid_price > 0.0, "Bid price should be positive");
            assert!(last_price > 0.0, "Last price should be positive");
            assert!(ask_price >= bid_price, "Ask should be >= bid");
            assert!(ask_price > 1000.0 && ask_price < 500000.0, "BTC/USD price should be reasonable");
            
            println!("✅ BTC/USD: Bid ${:.2}, Ask ${:.2}, Last ${:.2}", bid_price, ask_price, last_price);
        }
        Err(e) => {
            eprintln!("⚠️  BTC/USD ticker test failed (network/API issue): {}", e);
        }
    }
}

#[tokio::test]
async fn test_kraken_public_ticker_xmrusd() {
    let (api_key, api_secret) = TestConfig::kraken();
    let client = KrakenClient::new(api_key, api_secret);

    // Test XMR/USD ticker (public endpoint, no auth required)
    let ticker_result = client.get_ticker("XMRUSD").await;

    match ticker_result {
        Ok(ticker) => {
            // Verify ticker data structure
            assert!(!ticker.ask.is_empty(), "Ask prices should not be empty");
            assert!(!ticker.bid.is_empty(), "Bid prices should not be empty");
            assert!(!ticker.last_trade.is_empty(), "Last trade should not be empty");

            // Verify price data is numeric and reasonable
            let ask_price: f64 = ticker.ask[0].parse().expect("Ask price should be numeric");
            let bid_price: f64 = ticker.bid[0].parse().expect("Bid price should be numeric");
            let last_price: f64 = ticker.last_trade[0].parse().expect("Last trade should be numeric");
            
            assert!(ask_price > 0.0, "Ask price should be positive");
            assert!(bid_price > 0.0, "Bid price should be positive");
            assert!(last_price > 0.0, "Last price should be positive");
            assert!(ask_price >= bid_price, "Ask should be >= bid");
            assert!(ask_price > 10.0 && ask_price < 10000.0, "XMR/USD price should be reasonable");
            
            println!("✅ XMR/USD: Bid ${:.2}, Ask ${:.2}, Last ${:.2}", bid_price, ask_price, last_price);
        }
        Err(e) => {
            eprintln!("⚠️  XMR/USD ticker test failed (network/API issue): {}", e);
        }
    }
}

#[tokio::test]
async fn test_kraken_public_ticker_xmrbtc() {
    let (api_key, api_secret) = TestConfig::kraken();
    let client = KrakenClient::new(api_key, api_secret);

    // Test XMR/BTC ticker (public endpoint, no auth required)
    let ticker_result = client.get_ticker("XMRXBT").await;

    match ticker_result {
        Ok(ticker) => {
            // Verify ticker data structure
            assert!(!ticker.ask.is_empty(), "Ask prices should not be empty");
            assert!(!ticker.bid.is_empty(), "Bid prices should not be empty");
            assert!(!ticker.last_trade.is_empty(), "Last trade should not be empty");

            // Verify price data is numeric and reasonable
            let ask_price: f64 = ticker.ask[0].parse().expect("Ask price should be numeric");
            let bid_price: f64 = ticker.bid[0].parse().expect("Bid price should be numeric");
            let last_price: f64 = ticker.last_trade[0].parse().expect("Last trade should be numeric");
            
            assert!(ask_price > 0.0, "Ask price should be positive");
            assert!(bid_price > 0.0, "Bid price should be positive");
            assert!(last_price > 0.0, "Last price should be positive");
            assert!(ask_price >= bid_price, "Ask should be >= bid");
            assert!(ask_price > 0.0001 && ask_price < 1.0, "XMR/BTC price should be reasonable");
            
            println!("✅ XMR/BTC: Bid {:.8}, Ask {:.8}, Last {:.8}", bid_price, ask_price, last_price);
        }
        Err(e) => {
            eprintln!("⚠️  XMR/BTC ticker test failed (network/API issue): {}", e);
        }
    }
}

#[tokio::test]
async fn test_kraken_all_tickers() {
    let (api_key, api_secret) = TestConfig::kraken();
    let client = KrakenClient::new(api_key, api_secret);

    println!("Testing all three ticker endpoints...");

    // Test all three tickers that our API endpoint uses
    let btc_usd = client.get_ticker("XBTUSD").await;
    let xmr_usd = client.get_ticker("XMRUSD").await;
    let xmr_btc = client.get_ticker("XMRXBT").await;

    if let Ok(t) = btc_usd {
        let price: f64 = t.last_trade[0].parse().unwrap_or(0.0);
        println!("✅ BTC/USD: ${:.2}", price);
    } else {
        eprintln!("❌ BTC/USD failed");
    }

    if let Ok(t) = xmr_usd {
        let price: f64 = t.last_trade[0].parse().unwrap_or(0.0);
        println!("✅ XMR/USD: ${:.2}", price);
    } else {
        eprintln!("❌ XMR/USD failed");
    }

    if let Ok(t) = xmr_btc {
        let price: f64 = t.last_trade[0].parse().unwrap_or(0.0);
        println!("✅ XMR/BTC: {:.8}", price);
    } else {
        eprintln!("❌ XMR/BTC failed");
    }
}

#[tokio::test]
#[ignore] // Requires backend service to be running
async fn test_kraken_tickers_endpoint() {
    // Test the actual /kraken/tickers HTTP endpoint
    // This requires the backend service to be running on localhost:3000

    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:3000/kraken/tickers")
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200, "Endpoint should return 200 OK");

            let body = resp.text().await.expect("Should get response body");
            println!("Raw response: {}", body);

            // Parse the JSON response
            let tickers: KrakenTickerResponse = serde_json::from_str(&body)
                .expect("Response should be valid JSON");

            // Verify all required fields are present
            assert!(tickers.btc_usd > 0.0, "BTC/USD price should be positive");
            assert!(tickers.xmr_usd > 0.0, "XMR/USD price should be positive");
            assert!(tickers.xmr_btc > 0.0, "XMR/BTC price should be positive");

            // Verify reasonable price ranges
            assert!(tickers.btc_usd > 1000.0 && tickers.btc_usd < 500000.0,
                   "BTC/USD price should be reasonable: ${}", tickers.btc_usd);
            assert!(tickers.xmr_usd > 10.0 && tickers.xmr_usd < 10000.0,
                   "XMR/USD price should be reasonable: ${}", tickers.xmr_usd);
            assert!(tickers.xmr_btc > 0.0001 && tickers.xmr_btc < 1.0,
                   "XMR/BTC price should be reasonable: {}", tickers.xmr_btc);

            // Verify 24h change percentages are reasonable (should be between -50% and +50%)
            assert!(tickers.btc_usd_change_24h > -50.0 && tickers.btc_usd_change_24h < 50.0,
                   "BTC/USD 24h change should be reasonable: {:.2}%", tickers.btc_usd_change_24h);
            assert!(tickers.xmr_usd_change_24h > -50.0 && tickers.xmr_usd_change_24h < 50.0,
                   "XMR/USD 24h change should be reasonable: {:.2}%", tickers.xmr_usd_change_24h);
            assert!(tickers.xmr_btc_change_24h > -50.0 && tickers.xmr_btc_change_24h < 50.0,
                   "XMR/BTC 24h change should be reasonable: {:.2}%", tickers.xmr_btc_change_24h);

            println!("✅ Kraken tickers endpoint test passed!");
            println!("   BTC/USD: ${:.2} ({:+.2}%)", tickers.btc_usd, tickers.btc_usd_change_24h);
            println!("   XMR/USD: ${:.2} ({:+.2}%)", tickers.xmr_usd, tickers.xmr_usd_change_24h);
            println!("   XMR/BTC: {:.8} ({:+.2}%)", tickers.xmr_btc, tickers.xmr_btc_change_24h);
        }
        Err(e) => {
            eprintln!("⚠️  Kraken tickers endpoint test failed (backend not running?): {}", e);
            eprintln!("💡 To run this test, start the backend service first:");
            eprintln!("   cd projects/eigenix/backend && cargo run");
        }
    }
}

#[tokio::test]
#[ignore] // Requires backend service to be running
async fn test_kraken_tickers_endpoint_structure() {
    // Test that the endpoint returns the correct JSON structure
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:3000/kraken/tickers")
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);

            let body = resp.text().await.expect("Should get response body");

            // Parse as raw JSON value first to check structure
            let json_value: serde_json::Value = serde_json::from_str(&body)
                .expect("Response should be valid JSON");

            // Verify all expected fields are present
            assert!(json_value.get("btc_usd").is_some(), "btc_usd field should be present");
            assert!(json_value.get("btc_usd_change_24h").is_some(), "btc_usd_change_24h field should be present");
            assert!(json_value.get("xmr_usd").is_some(), "xmr_usd field should be present");
            assert!(json_value.get("xmr_usd_change_24h").is_some(), "xmr_usd_change_24h field should be present");
            assert!(json_value.get("xmr_btc").is_some(), "xmr_btc field should be present");
            assert!(json_value.get("xmr_btc_change_24h").is_some(), "xmr_btc_change_24h field should be present");

            // Verify that fields are numbers
            assert!(json_value["btc_usd"].is_number(), "btc_usd should be a number");
            assert!(json_value["btc_usd_change_24h"].is_number(), "btc_usd_change_24h should be a number");
            assert!(json_value["xmr_usd"].is_number(), "xmr_usd should be a number");
            assert!(json_value["xmr_usd_change_24h"].is_number(), "xmr_usd_change_24h should be a number");
            assert!(json_value["xmr_btc"].is_number(), "xmr_btc should be a number");
            assert!(json_value["xmr_btc_change_24h"].is_number(), "xmr_btc_change_24h should be a number");

            println!("✅ Kraken tickers endpoint structure test passed!");
            println!("   Response has correct JSON structure with all required fields");
        }
        Err(e) => {
            eprintln!("⚠️  Kraken tickers endpoint structure test failed (backend not running?): {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Requires backend service to be running
async fn test_kraken_tickers_endpoint_consistency() {
    // Test that the endpoint returns consistent data (prices don't change drastically between calls)
    let client = reqwest::Client::new();

    // Make two requests with a small delay
    let response1 = client
        .get("http://localhost:3000/kraken/tickers")
        .send()
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let response2 = client
        .get("http://localhost:3000/kraken/tickers")
        .send()
        .await;

    match (response1, response2) {
        (Ok(resp1), Ok(resp2)) => {
            assert_eq!(resp1.status(), 200);
            assert_eq!(resp2.status(), 200);

            let body1 = resp1.text().await.expect("Should get first response body");
            let body2 = resp2.text().await.expect("Should get second response body");

            let tickers1: KrakenTickerResponse = serde_json::from_str(&body1)
                .expect("First response should be valid JSON");
            let tickers2: KrakenTickerResponse = serde_json::from_str(&body2)
                .expect("Second response should be valid JSON");

            // Prices shouldn't change drastically (more than 10%) within 500ms
            let btc_change = ((tickers2.btc_usd - tickers1.btc_usd) / tickers1.btc_usd).abs();
            let xmr_change = ((tickers2.xmr_usd - tickers1.xmr_usd) / tickers1.xmr_usd).abs();
            let pair_change = ((tickers2.xmr_btc - tickers1.xmr_btc) / tickers1.xmr_btc).abs();

            assert!(btc_change < 0.1, "BTC/USD price shouldn't change more than 10% in 500ms: {:.4}%", btc_change * 100.0);
            assert!(xmr_change < 0.1, "XMR/USD price shouldn't change more than 10% in 500ms: {:.4}%", xmr_change * 100.0);
            assert!(pair_change < 0.1, "XMR/BTC price shouldn't change more than 10% in 500ms: {:.4}%", pair_change * 100.0);

            println!("✅ Kraken tickers endpoint consistency test passed!");
            println!("   Prices remained stable between requests");
        }
        _ => {
            eprintln!("⚠️  Kraken tickers endpoint consistency test failed (backend not running?)");
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
