use gloo_net::http::Request;
use serde::de::DeserializeOwned;

use crate::constants::api_base_url;

/// Shared API client with helper methods for making HTTP requests
pub struct ApiClient;

impl ApiClient {
    /// Make a GET request to the API
    pub async fn get<T: DeserializeOwned>(endpoint: &str) -> Result<T, String> {
        let url = format!("{}{}", api_base_url(), endpoint);
        
        Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<T>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Make a POST request to the API
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        endpoint: &str,
        body: &B,
    ) -> Result<T, String> {
        let url = format!("{}{}", api_base_url(), endpoint);
        
        Request::post(&url)
            .json(body)
            .map_err(|e| format!("Failed to serialize body: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<T>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Make a PUT request to the API
    pub async fn put<T: DeserializeOwned, B: serde::Serialize>(
        endpoint: &str,
        body: &B,
    ) -> Result<T, String> {
        let url = format!("{}{}", api_base_url(), endpoint);
        
        Request::put(&url)
            .json(body)
            .map_err(|e| format!("Failed to serialize body: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<T>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }
}

