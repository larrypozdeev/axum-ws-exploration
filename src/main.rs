use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};

#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/websocket", get(websocket_handler))
        .with_state(AppState {});

    let listener = tokio::net::TcpListener::bind("0.0.0.0:42069")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(mut socket: WebSocket, state: AppState) {
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



