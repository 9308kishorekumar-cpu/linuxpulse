use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/api/health", get(health))
}

async fn health() -> &'static str {
    "LinuxPulse backend is healthy"
}
