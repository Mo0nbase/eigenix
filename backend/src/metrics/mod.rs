//! Metrics collection and aggregation
//!
//! This module provides:
//! - Metric type definitions
//! - RPC clients for collecting metrics
//! - Background collector service

pub mod collector;
pub mod types;

// Re-export types for convenience
pub use types::*;
pub use collector::MetricsCollector;

