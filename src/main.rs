use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::{IntoResponse},
    routing::get,
    Router,
};


#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/websocket", get(websocket_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:42069")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(websocket)
}


async fn websocket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };

        if socket.send(msg).await.is_err() {
            return;
        }
    }
}





