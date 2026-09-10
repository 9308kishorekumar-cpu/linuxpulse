use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};

pub async fn websocket_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let _ = socket
        .send(Message::Text(
            "LinuxPulse WebSocket connected".into(),
        ))
        .await;
}
