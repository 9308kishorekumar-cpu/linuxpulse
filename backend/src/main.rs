mod api;
mod collectors;
mod insights;
mod system;
mod ws;

use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use axum::Router;
use system::collector::SnapshotCollector;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let shared_snapshot = Arc::new(RwLock::new(None));

    let collector_state = Arc::clone(&shared_snapshot);

    tokio::spawn(async move {
        let mut collector = SnapshotCollector::new();

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let snapshot = collector.collect();

            if let Ok(mut current) = collector_state.write() {
                *current = Some(snapshot.clone());
            }

            println!("CPU: {:.2}%", snapshot.cpu_usage_percent);
        }
    });

    let app: Router = api::router(shared_snapshot).layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind server");

    println!("LinuxPulse API listening on http://127.0.0.1:3000");

    axum::serve(listener, app).await.expect("server error");
}
