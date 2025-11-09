use gloo_net::http::Request;
use serde::de::DeserializeOwned;

use crate::constants::api_base_url;

/// Shared API client with helper methods for making HTTP requests
pub struct ApiClient;

impl ApiClient {
    /// Make a GET request to the API
    pub async fn get<T: DeserializeOwned>(endpoint: &str) -> Result<T, String> {
        let url = format!("{}{}", api_base_url(), endpoint);

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        // Log response status and text for debugging
        let status = response.status();
        let text = response.text().await
            .map_err(|e| format!("Failed to read response text: {}", e))?;

        dioxus_logger::tracing::info!("API GET {} -> Status: {}, Response: {}", url, status, text);

        if status < 200 || status >= 300 {
            return Err(format!("HTTP {}: {}", status, text));
        }

        if text.trim().is_empty() {
            return Err("Empty response from server".to_string());
        }

        serde_json::from_str::<T>(&text)
            .map_err(|e| format!("Failed to parse response '{}': {}", text, e))
    }

    /// Make a POST request to the API
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        endpoint: &str,
        body: &B,
    ) -> Result<T, String> {
        let url = format!("{}{}", api_base_url(), endpoint);

        let response = Request::post(&url)
            .json(body)
            .map_err(|e| format!("Failed to serialize body: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        // Log response status and text for debugging
        let status = response.status();
        let text = response.text().await
            .map_err(|e| format!("Failed to read response text: {}", e))?;

        dioxus_logger::tracing::info!("API POST {} -> Status: {}, Response: {}", url, status, text);

        if status < 200 || status >= 300 {
            return Err(format!("HTTP {}: {}", status, text));
        }

        if text.trim().is_empty() {
            return Err("Empty response from server".to_string());
        }

        serde_json::from_str::<T>(&text)
            .map_err(|e| format!("Failed to parse response '{}': {}", text, e))
    }

    /// Make a PUT request to the API
    pub async fn put<T: DeserializeOwned, B: serde::Serialize>(
        endpoint: &str,
        body: &B,
    ) -> Result<T, String> {
        let url = format!("{}{}", api_base_url(), endpoint);

        let response = Request::put(&url)
            .json(body)
            .map_err(|e| format!("Failed to serialize body: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        // Log response status and text for debugging
        let status = response.status();
        let text = response.text().await
            .map_err(|e| format!("Failed to read response text: {}", e))?;

        dioxus_logger::tracing::info!("API PUT {} -> Status: {}, Response: {}", url, status, text);

        if status < 200 || status >= 300 {
            return Err(format!("HTTP {}: {}", status, text));
        }

        if text.trim().is_empty() {
            return Err("Empty response from server".to_string());
        }

        serde_json::from_str::<T>(&text)
            .map_err(|e| format!("Failed to parse response '{}': {}", text, e))
    }
}

