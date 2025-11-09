/// API server port configuration
pub const API_PORT: u16 = 3000;

/// Base URL for API requests
pub fn api_base_url() -> String {
    format!("http://localhost:{}", API_PORT)
}
