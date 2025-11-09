/// Integration tests for ASB RPC client
///
/// Run with: cargo nextest run
/// Or just ASB tests: cargo nextest run asb
use eigenix_backend::services::AsbClient;

mod common;
use common::TestConfig;

#[tokio::test]
async fn test_asb_check_connection() {
    let url = TestConfig::asb_rpc_url();
    let client = AsbClient::new(url);
    assert!(client.check_connection().await.is_ok());
}

#[tokio::test]
async fn test_asb_get_bitcoin_balance() {
    let url = TestConfig::asb_rpc_url();
    let client = AsbClient::new(url);
    let balance = client.get_bitcoin_balance().await;
    assert!(balance.is_ok());
}

#[tokio::test]
async fn test_asb_get_bitcoin_seed() {
    let url = TestConfig::asb_rpc_url();
    let client = AsbClient::new(url);
    let descriptor = client.get_bitcoin_seed().await;
    assert!(descriptor.is_ok());
    assert!(!descriptor.unwrap().is_empty());
}

#[tokio::test]
async fn test_asb_get_monero_balance() {
    let url = TestConfig::asb_rpc_url();
    let client = AsbClient::new(url);
    let balance = client.get_monero_balance().await;
    assert!(balance.is_ok());
}

#[tokio::test]
async fn test_asb_get_monero_address() {
    let url = TestConfig::asb_rpc_url();
    let client = AsbClient::new(url);
    let address = client.get_monero_address().await;
    assert!(address.is_ok());
    let addr = address.unwrap();
    assert!(!addr.is_empty());
    assert!(addr.starts_with("4")); // Monero mainnet addresses start with 4
}

#[tokio::test]
async fn test_asb_get_monero_seed() {
    let url = TestConfig::asb_rpc_url();
    let client = AsbClient::new(url);
    let result = client.get_monero_seed().await;
    assert!(result.is_ok());

    let (seed, restore_height) = result.unwrap();
    let words: Vec<&str> = seed.split_whitespace().collect();
    assert_eq!(words.len(), 25, "Monero seed should be 25 words");
    assert!(
        restore_height > 0,
        "Restore height should be greater than 0"
    );
}

#[tokio::test]
async fn test_asb_get_multiaddresses() {
    let url = TestConfig::asb_rpc_url();
    let client = AsbClient::new(url);
    let addresses = client.get_multiaddresses().await;
    assert!(addresses.is_ok());
    assert!(!addresses.unwrap().is_empty());
}

#[tokio::test]
async fn test_asb_get_active_connections() {
    let url = TestConfig::asb_rpc_url();
    let client = AsbClient::new(url);
    let count = client.get_active_connections().await;
    assert!(count.is_ok());
}

#[tokio::test]
async fn test_asb_get_swaps() {
    let url = TestConfig::asb_rpc_url();
    let client = AsbClient::new(url);
    let swaps = client.get_swaps().await;
    assert!(swaps.is_ok());
}

#[tokio::test]
async fn test_asb_get_status() {
    let url = TestConfig::asb_rpc_url();
    let client = AsbClient::new(url);
    let status = client.get_status().await;
    assert!(status.is_ok());

    let status = status.unwrap();
    assert!(status.up, "ASB should be up");
}
