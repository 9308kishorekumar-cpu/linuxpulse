use std::time::Duration;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};

use crate::api::SharedSnapshot;

pub async fn websocket_handler(
    State(state): State<SharedSnapshot>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: SharedSnapshot) {
    loop {
        let snapshot = state
            .read()
            .ok()
            .and_then(|current| current.clone());

        if let Some(snapshot) = snapshot {
            match serde_json::to_string(&snapshot) {
                Ok(json) => {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
