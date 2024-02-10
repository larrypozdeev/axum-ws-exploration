use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, Response},
    routing::get,
    Router,
};
use futures::SinkExt;
use futures::StreamExt;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::broadcast;
use tokio::sync::Mutex;

struct AppState {
    user_set: Mutex<HashSet<String>>,
    tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    let user_set = Mutex::new(HashSet::new());
    let (tx, _rx) = broadcast::channel(100);

    let state = Arc::new(AppState { user_set, tx });

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

async fn websocket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    let mut username = String::new();
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(name) = message {
            let mut user_set = state.user_set.lock().await;

            if !user_set.contains(&name) {
                user_set.insert(name.to_owned());

                username.push_str(&name);
            }

            if !username.is_empty() {
                break;
            } else {
                // Only send our client that username is taken.
                let _ = sender
                    .send(Message::Text(String::from("Username already taken.")))
                    .await;

                return;
            }
        };
    }

    let mut rx = state.tx.subscribe();

    let msg = format!("{username} joined.");
    tracing::debug!("{msg}");
    // works, but doesn't keep the connection alive. broadcast is required
    // sender.send(Message::Text(msg.clone())).await.unwrap();
    state.tx.send(msg).unwrap();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // first part is completed

    // cloning things for receiving side

    let tx = state.tx.clone();
    let name = username.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Add username before message.
            let _ = tx.send(format!("{name}: {text}"));
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    // Send "user left" message (similar to "joined" above).
    let msg = format!("{username} left.");
    tracing::debug!("{msg}");
    let _ = state.tx.send(msg);

    // Remove username from map so new clients can take it again.
    state.user_set.lock().await.remove(&username);
}

async fn chat_handler() -> Html<&'static str> {
    Html(std::include_str!("../chat.html"))
}
