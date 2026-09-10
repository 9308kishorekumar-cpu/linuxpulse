mod collectors;
mod system;
mod api;

use std::{thread, time::Duration};

use axum::Router;
use system::collector::SnapshotCollector;


#[tokio::main]
async fn main() {
    tokio::spawn(async {
        let mut collector = SnapshotCollector::new();

        loop {
            thread::sleep(Duration::from_secs(1));

            let snapshot = collector.collect();

            println!("CPU: {:.2}%", snapshot.cpu_usage_percent);
        }
    });

    let app: Router = api::router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind server");

    println!("LinuxPulse API listening on http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
