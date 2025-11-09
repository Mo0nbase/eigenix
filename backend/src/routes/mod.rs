/// API route modules
///
/// This module organizes the API endpoints into logical groups:
/// - `bitcoin`: Endpoints for Bitcoin wallet operations
/// - `metrics`: Endpoints for retrieving system and service metrics
/// - `monero`: Endpoints for Monero wallet operations
/// - `wallets`: Combined wallet endpoints and orchestration
pub mod bitcoin;
pub mod metrics;
pub mod monero;
pub mod wallets;
