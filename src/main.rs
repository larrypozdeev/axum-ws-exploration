use futures::StreamExt;
use std::{collections::HashSet, sync::Arc};

use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade, Message},
        State,
    },
    response::{Html, Response},
    routing::get,
    Router,
};
use tokio::sync::Mutex;

struct AppState {
    user_set: Mutex<HashSet<String>>,
}

#[tokio::main]
async fn main() {
    let user_set = Mutex::new(HashSet::new());
    let state = Arc::new(AppState { user_set });

    let app = Router::new()
        .route("/chat", get(chat_handler))
        .route("/websocket", get(websocket_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:42069")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(mut socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    let username = String::new();
    while let Some(Ok(message)) = receiver.next().await {
       // print out the message
        println!("{:?}", message);

    }
}

async fn chat_handler() -> Html<&'static str> {
    Html(std::include_str!("../chat.html"))
}
