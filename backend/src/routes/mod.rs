/// API route modules
///
/// This module organizes the API endpoints into logical groups:
/// - `bitcoin`: Endpoints for Bitcoin wallet operations
/// - `kraken`: Endpoints for Kraken exchange data
/// - `metrics`: Endpoints for retrieving system and service metrics
/// - `monero`: Endpoints for Monero wallet operations
/// - `trading`: Endpoints for trading engine control and monitoring
/// - `wallets`: Combined wallet endpoints and orchestration
pub mod bitcoin;
pub mod kraken;
pub mod metrics;
pub mod monero;
pub mod trading;
pub mod wallets;
