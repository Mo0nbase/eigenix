/// Backend API configuration
///
/// The backend server typically runs on port 3000 by default.
/// Update these values if your backend is configured differently.

/// API server port configuration
pub const API_PORT: u16 = 3000;

/// API server host configuration
pub const API_HOST: &str = "nixlab";

/// Base URL for API requests
pub fn api_base_url() -> String {
    format!("http://{}:{}", API_HOST, API_PORT)
}
