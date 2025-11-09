use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

use crate::{db, AppState};

/// Query parameters for historical metrics
#[derive(Deserialize)]
pub struct HistoryQuery {
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
}

/// Query parameters for container history
#[derive(Deserialize)]
pub struct ContainerHistoryQuery {
    name: String,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
}

/// Query parameters for interval metrics
#[derive(Deserialize)]
pub struct IntervalQuery {
    minutes: Option<i64>,
}

/// Get latest Bitcoin metrics
pub async fn bitcoin_metrics(
    State(state): State<AppState>,
) -> Result<Json<db::StoredBitcoinMetrics>, String> {
    let metrics = state
        .db
        .get_latest_bitcoin_metrics()
        .await
        .map_err(|e| format!("Failed to get Bitcoin metrics: {}", e))?
        .ok_or_else(|| "No Bitcoin metrics available".to_string())?;

    Ok(Json(metrics))
}

/// Get latest Monero metrics
pub async fn monero_metrics(
    State(state): State<AppState>,
) -> Result<Json<db::StoredMoneroMetrics>, String> {
    let metrics = state
        .db
        .get_latest_monero_metrics()
        .await
        .map_err(|e| format!("Failed to get Monero metrics: {}", e))?
        .ok_or_else(|| "No Monero metrics available".to_string())?;

    Ok(Json(metrics))
}

/// Get latest ASB metrics
pub async fn asb_metrics(State(state): State<AppState>) -> Result<Json<db::StoredAsbMetrics>, String> {
    let metrics = state
        .db
        .get_latest_asb_metrics()
        .await
        .map_err(|e| format!("Failed to get ASB metrics: {}", e))?
        .ok_or_else(|| "No ASB metrics available".to_string())?;

    Ok(Json(metrics))
}

/// Get latest Electrs metrics
pub async fn electrs_metrics(
    State(state): State<AppState>,
) -> Result<Json<db::StoredElectrsMetrics>, String> {
    let metrics = state
        .db
        .get_latest_electrs_metrics()
        .await
        .map_err(|e| format!("Failed to get Electrs metrics: {}", e))?
        .ok_or_else(|| "No Electrs metrics available".to_string())?;

    Ok(Json(metrics))
}

/// Get latest container metrics
pub async fn container_metrics(
    State(state): State<AppState>,
) -> Result<Json<Vec<db::StoredContainerMetrics>>, String> {
    let metrics = state
        .db
        .get_latest_container_metrics()
        .await
        .map_err(|e| format!("Failed to get container metrics: {}", e))?;

    Ok(Json(metrics))
}

/// Get metrics summary
pub async fn summary_metrics(
    State(state): State<AppState>,
) -> Result<Json<db::MetricsSummary>, String> {
    let summary = state
        .db
        .get_summary()
        .await
        .map_err(|e| format!("Failed to get metrics summary: {}", e))?;

    Ok(Json(summary))
}

/// Get Bitcoin metrics history
pub async fn bitcoin_history(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<db::StoredBitcoinMetrics>>, String> {
    let to = query.to.unwrap_or_else(Utc::now);
    let from = query.from.unwrap_or_else(|| to - Duration::hours(24));

    let history = state
        .db
        .get_bitcoin_history(from, to)
        .await
        .map_err(|e| format!("Failed to get Bitcoin history: {}", e))?;

    Ok(Json(history))
}

/// Get Monero metrics history
pub async fn monero_history(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<db::StoredMoneroMetrics>>, String> {
    let to = query.to.unwrap_or_else(Utc::now);
    let from = query.from.unwrap_or_else(|| to - Duration::hours(24));

    let history = state
        .db
        .get_monero_history(from, to)
        .await
        .map_err(|e| format!("Failed to get Monero history: {}", e))?;

    Ok(Json(history))
}

/// Get ASB metrics history
pub async fn asb_history(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<db::StoredAsbMetrics>>, String> {
    let to = query.to.unwrap_or_else(Utc::now);
    let from = query.from.unwrap_or_else(|| to - Duration::hours(24));

    let history = state
        .db
        .get_asb_history(from, to)
        .await
        .map_err(|e| format!("Failed to get ASB history: {}", e))?;

    Ok(Json(history))
}

/// Get Electrs metrics history
pub async fn electrs_history(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<db::StoredElectrsMetrics>>, String> {
    let to = query.to.unwrap_or_else(Utc::now);
    let from = query.from.unwrap_or_else(|| to - Duration::hours(24));

    let history = state
        .db
        .get_electrs_history(from, to)
        .await
        .map_err(|e| format!("Failed to get Electrs history: {}", e))?;

    Ok(Json(history))
}

/// Get container metrics history
pub async fn container_history(
    State(state): State<AppState>,
    Query(query): Query<ContainerHistoryQuery>,
) -> Result<Json<Vec<db::StoredContainerMetrics>>, String> {
    let to = query.to.unwrap_or_else(Utc::now);
    let from = query.from.unwrap_or_else(|| to - Duration::hours(24));

    let history = state
        .db
        .get_container_history(&query.name, from, to)
        .await
        .map_err(|e| format!("Failed to get container history: {}", e))?;

    Ok(Json(history))
}

/// Get Bitcoin metrics for time interval
pub async fn bitcoin_interval(
    State(state): State<AppState>,
    Query(query): Query<IntervalQuery>,
) -> Result<Json<Vec<db::StoredBitcoinMetrics>>, String> {
    let minutes = query.minutes.unwrap_or(5);
    let to = Utc::now();
    let from = to - Duration::minutes(minutes);

    let history = state
        .db
        .get_bitcoin_history(from, to)
        .await
        .map_err(|e| format!("Failed to get Bitcoin interval metrics: {}", e))?;

    Ok(Json(history))
}

/// Get Monero metrics for time interval
pub async fn monero_interval(
    State(state): State<AppState>,
    Query(query): Query<IntervalQuery>,
) -> Result<Json<Vec<db::StoredMoneroMetrics>>, String> {
    let minutes = query.minutes.unwrap_or(5);
    let to = Utc::now();
    let from = to - Duration::minutes(minutes);

    let history = state
        .db
        .get_monero_history(from, to)
        .await
        .map_err(|e| format!("Failed to get Monero interval metrics: {}", e))?;

    Ok(Json(history))
}

/// Get ASB metrics for time interval
pub async fn asb_interval(
    State(state): State<AppState>,
    Query(query): Query<IntervalQuery>,
) -> Result<Json<Vec<db::StoredAsbMetrics>>, String> {
    let minutes = query.minutes.unwrap_or(5);
    let to = Utc::now();
    let from = to - Duration::minutes(minutes);

    let history = state
        .db
        .get_asb_history(from, to)
        .await
        .map_err(|e| format!("Failed to get ASB interval metrics: {}", e))?;

    Ok(Json(history))
}

/// Get Electrs metrics for time interval
pub async fn electrs_interval(
    State(state): State<AppState>,
    Query(query): Query<IntervalQuery>,
) -> Result<Json<Vec<db::StoredElectrsMetrics>>, String> {
    let minutes = query.minutes.unwrap_or(5);
    let to = Utc::now();
    let from = to - Duration::minutes(minutes);

    let history = state
        .db
        .get_electrs_history(from, to)
        .await
        .map_err(|e| format!("Failed to get Electrs interval metrics: {}", e))?;

    Ok(Json(history))
}

/// Create the metrics routes router
pub fn metrics_routes() -> Router<AppState> {
    Router::new()
        .route("/summary", get(summary_metrics))
        .route("/bitcoin", get(bitcoin_metrics))
        .route("/bitcoin/history", get(bitcoin_history))
        .route("/bitcoin/interval", get(bitcoin_interval))
        .route("/monero", get(monero_metrics))
        .route("/monero/history", get(monero_history))
        .route("/monero/interval", get(monero_interval))
        .route("/asb", get(asb_metrics))
        .route("/asb/history", get(asb_history))
        .route("/asb/interval", get(asb_interval))
        .route("/electrs", get(electrs_metrics))
        .route("/electrs/history", get(electrs_history))
        .route("/electrs/interval", get(electrs_interval))
        .route("/containers", get(container_metrics))
        .route("/containers/history", get(container_history))
}
