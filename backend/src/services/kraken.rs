use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;

type HmacSha512 = Hmac<Sha512>;

const KRAKEN_API_URL: &str = "https://api.kraken.com";

/// Kraken API client for trading
///
/// API keys can have different permissions configured in the Kraken dashboard.
/// For testing, create API keys with limited permissions (query only, no trading/withdrawals).
///
/// # Environment Variables for Testing
/// - KRAKEN_API_KEY: Kraken API key
/// - KRAKEN_API_SECRET: Kraken API secret
pub struct KrakenClient {
    api_key: String,
    api_secret: String,
    client: reqwest::Client,
}

/// Kraken API error response
#[derive(Debug, Deserialize)]
struct KrakenErrorResponse {
    error: Vec<String>,
}

/// Kraken API response wrapper
#[derive(Debug, Deserialize)]
struct KrakenResponse<T> {
    error: Vec<String>,
    result: Option<T>,
}

/// Ticker information
#[derive(Debug, Deserialize, Serialize)]
pub struct TickerInfo {
    #[serde(rename = "a")]
    pub ask: Vec<String>, // [price, whole lot volume, lot volume]
    #[serde(rename = "b")]
    pub bid: Vec<String>, // [price, whole lot volume, lot volume]
    #[serde(rename = "c")]
    pub last_trade: Vec<String>, // [price, lot volume]
    #[serde(rename = "v")]
    pub volume: Vec<String>, // [today, last 24 hours]
    #[serde(rename = "p")]
    pub vwap: Vec<String>, // [today, last 24 hours]
    #[serde(rename = "o")]
    pub open: String, // Today's opening price
}

/// Order information
#[derive(Debug, Deserialize, Serialize)]
pub struct OrderInfo {
    pub txid: Vec<String>, // Transaction IDs
    pub descr: OrderDescription,
}

/// Order description
#[derive(Debug, Deserialize, Serialize)]
pub struct OrderDescription {
    pub order: String,
    pub close: Option<String>,
}

/// Order status
#[derive(Debug, Deserialize, Serialize)]
pub struct OrderStatus {
    pub status: String,
    pub opentm: f64,
    pub closetm: Option<f64>,
    pub vol: String,
    pub vol_exec: String,
    pub cost: String,
    pub fee: String,
    pub price: String,
    pub descr: OrderStatusDescription,
}

/// Order status description
#[derive(Debug, Deserialize, Serialize)]
pub struct OrderStatusDescription {
    pub pair: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub ordertype: String,
    pub price: String,
    pub price2: String,
}

/// Deposit address information
#[derive(Debug, Deserialize, Serialize)]
pub struct DepositAddress {
    pub address: String,
    pub expiretm: Option<String>,
    pub new: Option<bool>,
    pub memo: Option<String>, // For assets that require a memo/tag
}

/// Deposit method information
#[derive(Debug, Deserialize, Serialize)]
pub struct DepositMethod {
    pub method: String,
    pub limit: Option<String>,
    pub fee: Option<String>,
    #[serde(rename = "gen-address")]
    pub gen_address: Option<bool>,
}

/// Withdrawal information
#[derive(Debug, Deserialize, Serialize)]
pub struct WithdrawalInfo {
    pub refid: String, // Reference ID for the withdrawal
}

/// Deposit status
#[derive(Debug, Deserialize, Serialize)]
pub struct DepositStatus {
    pub method: String,
    pub aclass: String,
    pub asset: String,
    pub refid: String,
    pub txid: String,
    pub info: String,
    pub amount: String,
    pub fee: Option<String>,
    pub time: u64,
    pub status: String,
}

/// Withdrawal status
#[derive(Debug, Deserialize, Serialize)]
pub struct WithdrawalStatus {
    pub method: String,
    pub aclass: String,
    pub asset: String,
    pub refid: String,
    pub txid: String,
    pub info: String,
    pub amount: String,
    pub fee: String,
    pub time: u64,
    pub status: String,
}

impl KrakenClient {
    /// Create a new Kraken API client
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            api_key,
            api_secret,
            client: reqwest::Client::new(),
        }
    }

    /// Generate API signature for authenticated requests
    fn generate_signature(&self, url_path: &str, nonce: u64, postdata: &str) -> Result<String> {
        // Decode base64 secret
        let secret = general_purpose::STANDARD
            .decode(&self.api_secret)
            .context("Failed to decode API secret")?;

        // Create SHA256 hash of (nonce + postdata)
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}", nonce, postdata));
        let hash = hasher.finalize();

        // Create message: url_path + hash
        let mut message = url_path.as_bytes().to_vec();
        message.extend_from_slice(&hash);

        // Create HMAC-SHA512
        let mut mac = HmacSha512::new_from_slice(&secret).context("Failed to create HMAC")?;
        mac.update(&message);
        let result = mac.finalize();

        // Encode to base64
        Ok(general_purpose::STANDARD.encode(result.into_bytes()))
    }

    /// Make a public API request (no authentication)
    async fn public_request<T>(&self, endpoint: &str, params: &[(&str, &str)]) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/0/public/{}", KRAKEN_API_URL, endpoint);

        let response = self
            .client
            .get(&url)
            .query(params)
            .send()
            .await
            .context("Failed to send request")?;

        let kraken_response: KrakenResponse<T> =
            response.json().await.context("Failed to parse response")?;

        if !kraken_response.error.is_empty() {
            anyhow::bail!("Kraken API error: {:?}", kraken_response.error);
        }

        kraken_response.result.context("Missing result in response")
    }

    /// Make a private API request (with authentication)
    async fn private_request<T>(
        &self,
        endpoint: &str,
        params: &mut HashMap<String, String>,
    ) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis() as u64;

        params.insert("nonce".to_string(), nonce.to_string());

        let url_path = format!("/0/private/{}", endpoint);
        let url = format!("{}{}", KRAKEN_API_URL, url_path);

        // Build POST data
        let postdata: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        // Generate signature
        let signature = self.generate_signature(&url_path, nonce, &postdata)?;

        let response = self
            .client
            .post(&url)
            .header("API-Key", &self.api_key)
            .header("API-Sign", signature)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(postdata)
            .send()
            .await
            .context("Failed to send request")?;

        let kraken_response: KrakenResponse<T> =
            response.json().await.context("Failed to parse response")?;

        if !kraken_response.error.is_empty() {
            anyhow::bail!("Kraken API error: {:?}", kraken_response.error);
        }

        kraken_response.result.context("Missing result in response")
    }

    /// Get ticker information for a trading pair
    /// Example: get_ticker("XBTXMR") for BTC/XMR pair
    pub async fn get_ticker(&self, pair: &str) -> Result<TickerInfo> {
        let result: HashMap<String, TickerInfo> =
            self.public_request("Ticker", &[("pair", pair)]).await?;

        result
            .into_iter()
            .next()
            .map(|(_, info)| info)
            .context("No ticker info returned")
    }

    /// Get account balance
    pub async fn get_balance(&self) -> Result<HashMap<String, String>> {
        self.private_request("Balance", &mut HashMap::new()).await
    }

    /// Place a market order to trade BTC for XMR
    ///
    /// # Arguments
    /// * `volume` - Amount of BTC to sell (e.g., "0.01" for 0.01 BTC)
    ///
    /// # Returns
    /// Order information including transaction ID
    pub async fn trade_btc_for_xmr(&self, volume: &str) -> Result<OrderInfo> {
        self.place_order("XBTXMR", "sell", "market", volume, None)
            .await
    }

    /// Place a limit order to trade BTC for XMR at a specific price
    ///
    /// # Arguments
    /// * `volume` - Amount of BTC to sell
    /// * `price` - Limit price in XMR per BTC
    pub async fn trade_btc_for_xmr_limit(&self, volume: &str, price: &str) -> Result<OrderInfo> {
        self.place_order("XBTXMR", "sell", "limit", volume, Some(price))
            .await
    }

    /// Place an order on Kraken
    ///
    /// # Arguments
    /// * `pair` - Asset pair (e.g., "XBTXMR" for BTC/XMR)
    /// * `type_` - Order type: "buy" or "sell"
    /// * `ordertype` - Order type: "market" or "limit"
    /// * `volume` - Order volume
    /// * `price` - Price (required for limit orders)
    pub async fn place_order(
        &self,
        pair: &str,
        type_: &str,
        ordertype: &str,
        volume: &str,
        price: Option<&str>,
    ) -> Result<OrderInfo> {
        let mut params = HashMap::new();
        params.insert("pair".to_string(), pair.to_string());
        params.insert("type".to_string(), type_.to_string());
        params.insert("ordertype".to_string(), ordertype.to_string());
        params.insert("volume".to_string(), volume.to_string());

        if let Some(p) = price {
            params.insert("price".to_string(), p.to_string());
        }

        self.private_request("AddOrder", &mut params).await
    }

    /// Query order status
    ///
    /// # Arguments
    /// * `txid` - Transaction ID from order placement
    pub async fn query_order(&self, txid: &str) -> Result<HashMap<String, OrderStatus>> {
        let mut params = HashMap::new();
        params.insert("txid".to_string(), txid.to_string());

        self.private_request("QueryOrders", &mut params).await
    }

    /// Cancel an order
    ///
    /// # Arguments
    /// * `txid` - Transaction ID of the order to cancel
    pub async fn cancel_order(&self, txid: &str) -> Result<HashMap<String, String>> {
        let mut params = HashMap::new();
        params.insert("txid".to_string(), txid.to_string());

        self.private_request("CancelOrder", &mut params).await
    }

    // ===== Deposit and Withdrawal Methods =====

    /// Get deposit methods for an asset
    ///
    /// # Arguments
    /// * `asset` - Asset to get deposit methods for (e.g., "XBT" for Bitcoin, "XMR" for Monero)
    pub async fn get_deposit_methods(&self, asset: &str) -> Result<Vec<DepositMethod>> {
        let mut params = HashMap::new();
        params.insert("asset".to_string(), asset.to_string());

        self.private_request("DepositMethods", &mut params).await
    }

    /// Get or generate a deposit address for an asset
    ///
    /// # Arguments
    /// * `asset` - Asset to get deposit address for (e.g., "XBT" for Bitcoin, "XMR" for Monero)
    /// * `method` - Deposit method name (get from get_deposit_methods)
    /// * `new` - Whether to generate a new address (default: false)
    pub async fn get_deposit_address(
        &self,
        asset: &str,
        method: &str,
        new: bool,
    ) -> Result<Vec<DepositAddress>> {
        let mut params = HashMap::new();
        params.insert("asset".to_string(), asset.to_string());
        params.insert("method".to_string(), method.to_string());
        if new {
            params.insert("new".to_string(), "true".to_string());
        }

        self.private_request("DepositAddresses", &mut params).await
    }

    /// Get Bitcoin deposit address
    ///
    /// # Arguments
    /// * `new` - Whether to generate a new address
    pub async fn get_btc_deposit_address(&self, new: bool) -> Result<String> {
        // First get deposit methods to find the right method name
        let methods = self.get_deposit_methods("XBT").await?;
        let method = methods
            .first()
            .context("No deposit methods available for Bitcoin")?;

        let addresses = self.get_deposit_address("XBT", &method.method, new).await?;
        let addr = addresses.first().context("No deposit address returned")?;

        Ok(addr.address.clone())
    }

    /// Get Monero deposit address
    ///
    /// # Arguments
    /// * `new` - Whether to generate a new address
    pub async fn get_xmr_deposit_address(&self, new: bool) -> Result<String> {
        // First get deposit methods to find the right method name
        let methods = self.get_deposit_methods("XMR").await?;
        let method = methods
            .first()
            .context("No deposit methods available for Monero")?;

        let addresses = self.get_deposit_address("XMR", &method.method, new).await?;
        let addr = addresses.first().context("No deposit address returned")?;

        Ok(addr.address.clone())
    }

    /// Withdraw funds from Kraken
    ///
    /// # Arguments
    /// * `asset` - Asset to withdraw (e.g., "XBT" for Bitcoin, "XMR" for Monero)
    /// * `key` - Withdrawal key name (must be pre-configured in Kraken account)
    /// * `amount` - Amount to withdraw
    pub async fn withdraw(&self, asset: &str, key: &str, amount: &str) -> Result<WithdrawalInfo> {
        let mut params = HashMap::new();
        params.insert("asset".to_string(), asset.to_string());
        params.insert("key".to_string(), key.to_string());
        params.insert("amount".to_string(), amount.to_string());

        self.private_request("Withdraw", &mut params).await
    }

    /// Withdraw Bitcoin to a pre-configured address
    ///
    /// # Arguments
    /// * `key` - Withdrawal key name configured in Kraken account
    /// * `amount` - Amount of BTC to withdraw
    pub async fn withdraw_btc(&self, key: &str, amount: &str) -> Result<WithdrawalInfo> {
        self.withdraw("XBT", key, amount).await
    }

    /// Withdraw Monero to a pre-configured address
    ///
    /// # Arguments
    /// * `key` - Withdrawal key name configured in Kraken account
    /// * `amount` - Amount of XMR to withdraw
    pub async fn withdraw_xmr(&self, key: &str, amount: &str) -> Result<WithdrawalInfo> {
        self.withdraw("XMR", key, amount).await
    }

    /// Get status of recent deposits
    ///
    /// # Arguments
    /// * `asset` - Optional asset filter (e.g., "XBT", "XMR")
    pub async fn get_deposit_status(&self, asset: Option<&str>) -> Result<Vec<DepositStatus>> {
        let mut params = HashMap::new();
        if let Some(a) = asset {
            params.insert("asset".to_string(), a.to_string());
        }

        self.private_request("DepositStatus", &mut params).await
    }

    /// Get status of recent withdrawals
    ///
    /// # Arguments
    /// * `asset` - Optional asset filter (e.g., "XBT", "XMR")
    pub async fn get_withdrawal_status(
        &self,
        asset: Option<&str>,
    ) -> Result<Vec<WithdrawalStatus>> {
        let mut params = HashMap::new();
        if let Some(a) = asset {
            params.insert("asset".to_string(), a.to_string());
        }

        self.private_request("WithdrawStatus", &mut params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access and can be flaky
    async fn test_get_ticker() {
        let client = KrakenClient::new("".to_string(), "".to_string());
        let ticker = client.get_ticker("XBTXMR").await;
        // Don't assert success - this is a basic smoke test
        // Proper testing is done in integration tests
        match ticker {
            Ok(_) => println!("Ticker fetch succeeded"),
            Err(e) => eprintln!("Ticker fetch failed (expected in some environments): {}", e),
        }
    }
}
