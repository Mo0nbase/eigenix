use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;

type HmacSha512 = Hmac<Sha512>;

const KRAKEN_API_URL: &str = "https://api.kraken.com";

/// Kraken API client for trading
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run with valid API credentials
    async fn test_get_ticker() {
        let client = KrakenClient::new("".to_string(), "".to_string());
        let ticker = client.get_ticker("XBTXMR").await;
        assert!(ticker.is_ok());
    }
}
