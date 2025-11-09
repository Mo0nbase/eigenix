/// Integration tests for Bitcoin wallet
///
/// Run with: cargo nextest run bitcoin_wallet
///
/// Note: These tests require the wallet to be initialized first.
/// Run the backend once to initialize: cargo run
use eigenix_backend::wallets::BitcoinWallet;

mod common;
use common::TestConfig;

#[tokio::test]
async fn test_bitcoin_wallet_connect() {
    let (rpc_url, cookie_path, wallet_name) = TestConfig::bitcoin_wallet();
    let wallet = BitcoinWallet::connect_existing(rpc_url, &cookie_path, &wallet_name).await;

    if wallet.is_err() {
        eprintln!("⚠️  Bitcoin wallet not initialized. Run backend first: cargo run");
    }

    // Don't fail the test if wallet doesn't exist yet
    // This allows tests to run before initialization
}

#[tokio::test]
#[ignore] // Only run with --ignored flag when wallet is initialized
async fn test_bitcoin_wallet_get_balance() {
    let (rpc_url, cookie_path, wallet_name) = TestConfig::bitcoin_wallet();
    let wallet = BitcoinWallet::connect_existing(rpc_url, &cookie_path, &wallet_name)
        .await
        .expect("Wallet should be initialized");

    let balance = wallet.get_balance().await;
    assert!(balance.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_bitcoin_wallet_get_new_address() {
    let (rpc_url, cookie_path, wallet_name) = TestConfig::bitcoin_wallet();
    let wallet = BitcoinWallet::connect_existing(rpc_url, &cookie_path, &wallet_name)
        .await
        .expect("Wallet should be initialized");

    let address = wallet.get_new_address(Some("test")).await;
    assert!(address.is_ok());

    let addr = address.unwrap();
    assert!(!addr.is_empty());
    assert!(addr.starts_with("bc1") || addr.starts_with("1") || addr.starts_with("3"));
}

#[tokio::test]
#[ignore]
async fn test_bitcoin_wallet_validate_address() {
    let (rpc_url, cookie_path, wallet_name) = TestConfig::bitcoin_wallet();
    let wallet = BitcoinWallet::connect_existing(rpc_url, &cookie_path, &wallet_name)
        .await
        .expect("Wallet should be initialized");

    // Valid mainnet address
    let valid = wallet
        .validate_address("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq")
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
async fn test_bitcoin_wallet_list_transactions() {
    let (rpc_url, cookie_path, wallet_name) = TestConfig::bitcoin_wallet();
    let wallet = BitcoinWallet::connect_existing(rpc_url, &cookie_path, &wallet_name)
        .await
        .expect("Wallet should be initialized");

    let txs = wallet.list_transactions(10).await;
    assert!(txs.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_bitcoin_wallet_is_ready() {
    let (rpc_url, cookie_path, wallet_name) = TestConfig::bitcoin_wallet();
    let wallet = BitcoinWallet::connect_existing(rpc_url, &cookie_path, &wallet_name)
        .await
        .expect("Wallet should be initialized");

    assert!(wallet.is_ready().await);
}
