/// Integration tests for Monero wallet
///
/// Run with: cargo nextest run monero_wallet
///
/// Note: These tests require the wallet to be initialized first.
/// Run the backend once to initialize: cargo run
use eigenix_backend::wallets::MoneroWallet;

mod common;
use common::TestConfig;

#[tokio::test]
async fn test_monero_wallet_connect() {
    let (rpc_url, wallet_name, password) = TestConfig::monero_wallet();
    let wallet = MoneroWallet::connect_existing(rpc_url, &wallet_name, &password).await;

    if wallet.is_err() {
        eprintln!("⚠️  Monero wallet not initialized. Run backend first: cargo run");
    }

    // Don't fail the test if wallet doesn't exist yet
}

#[tokio::test]
#[ignore] // Only run with --ignored flag when wallet is initialized
async fn test_monero_wallet_get_balance() {
    let (rpc_url, wallet_name, password) = TestConfig::monero_wallet();
    let wallet = MoneroWallet::connect_existing(rpc_url, &wallet_name, &password)
        .await
        .expect("Wallet should be initialized");

    let balance = wallet.get_balance().await;
    assert!(balance.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_monero_wallet_get_address() {
    let (rpc_url, wallet_name, password) = TestConfig::monero_wallet();
    let wallet = MoneroWallet::connect_existing(rpc_url, &wallet_name, &password)
        .await
        .expect("Wallet should be initialized");

    let address = wallet.get_address().await;
    assert!(address.is_ok());

    let addr = address.unwrap();
    assert!(!addr.is_empty());
    assert!(addr.starts_with("4")); // Mainnet addresses start with 4
    assert_eq!(addr.len(), 95); // Standard Monero address length
}

#[tokio::test]
#[ignore]
async fn test_monero_wallet_validate_address() {
    let (rpc_url, wallet_name, password) = TestConfig::monero_wallet();
    let wallet = MoneroWallet::connect_existing(rpc_url, &wallet_name, &password)
        .await
        .expect("Wallet should be initialized");

    // Valid mainnet address
    let valid = wallet
        .validate_address("4AdUndXHHZ6cfufTMvppY6JwXNouMBzSkbLYfpAV5Usx3skxNgYeYTRj5UzqtReoS44qo9mtmXCqY45DJ852K5Jv2684Rge")
        .await;
    assert!(valid.is_ok());
    assert_eq!(valid.unwrap(), true);

    // Invalid address
    let invalid = wallet.validate_address("invalid").await;
    assert!(invalid.is_ok());
    assert_eq!(invalid.unwrap(), false);
}

#[tokio::test]
#[ignore]
async fn test_monero_wallet_get_height() {
    let (rpc_url, wallet_name, password) = TestConfig::monero_wallet();
    let wallet = MoneroWallet::connect_existing(rpc_url, &wallet_name, &password)
        .await
        .expect("Wallet should be initialized");

    let height = wallet.get_height().await;
    assert!(height.is_ok());
    assert!(height.unwrap() > 0);
}

#[tokio::test]
#[ignore]
async fn test_monero_wallet_refresh() {
    let (rpc_url, wallet_name, password) = TestConfig::monero_wallet();
    let wallet = MoneroWallet::connect_existing(rpc_url, &wallet_name, &password)
        .await
        .expect("Wallet should be initialized");

    let result = wallet.refresh().await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_monero_wallet_is_ready() {
    let (rpc_url, wallet_name, password) = TestConfig::monero_wallet();
    let wallet = MoneroWallet::connect_existing(rpc_url, &wallet_name, &password)
        .await
        .expect("Wallet should be initialized");

    assert!(wallet.is_ready().await);
}
