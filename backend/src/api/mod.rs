use axum::{extract::State, routing::get, Json, Router};

use crate::system::snapshot::SystemSnapshot;
use crate::insights::analyze;

use std::sync::{Arc, RwLock};

pub type SharedSnapshot = Arc<RwLock<Option<SystemSnapshot>>>;

pub fn router(state: SharedSnapshot) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/system", get(system))
        .route("/api/insights", get(insights))
        .route("/api/ws", get(crate::ws::websocket_handler))
        .with_state(state)
}

async fn health() -> &'static str {
    "LinuxPulse backend is healthy"
}

async fn system(
    State(state): State<SharedSnapshot>,
) -> Json<Option<SystemSnapshot>> {
    let snapshot = state
        .read()
        .ok()
        .and_then(|current| current.clone());

    Json(snapshot)
}

async fn insights(
    State(state): State<SharedSnapshot>,
) -> Json<Vec<crate::insights::Insight>> {
    let snapshot = state
        .read()
        .ok()
        .and_then(|current| current.clone());

    match snapshot {
        Some(snapshot) => Json(analyze(&snapshot)),
        None => Json(Vec::new()),
    }
}
