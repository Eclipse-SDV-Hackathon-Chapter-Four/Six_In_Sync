use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Html,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use std::{
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::broadcast,
};

#[derive(Clone, Serialize, Debug)]
struct StateUpdate {
    r#type: String,
    state: String,
    child: bool,
    temp: String,
    window: String,
    log_text: Option<String>,
    log_level: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Broadcast channel for streaming events to all WebSocket clients
    let (tx, _rx) = broadcast::channel::<String>(100);
    let tx = Arc::new(tx);

    // Spawn Guardian Process Reader thread
    let tx_guardian = tx.clone();
    tokio::spawn(async move {
        run_guardian_wrapper(tx_guardian).await;
    });

    // Build Axum web server router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(move |ws| ws_handler(ws, tx.clone())));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind port 8080");

    println!("==================================================");
    println!("🚀 Rust Safety Dashboard Running on http://localhost:8080");
    println!("==================================================");

    axum::serve(listener, app).await.unwrap();
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../../web_dashboard.html"))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    tx: Arc<broadcast::Sender<String>>,
) -> axum::response::Response {
    ws.on_upgrade(|socket| handle_socket(socket, tx))
}

async fn handle_socket(socket: WebSocket, tx: Arc<broadcast::Sender<String>>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(_)) = receiver.next().await {}
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

async fn run_guardian_wrapper(tx: Arc<broadcast::Sender<String>>) {
    let child_res = Command::new("./target/debug/guardian")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child_res {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to start ./target/debug/guardian: {}", e);
            return;
        }
    };

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout).lines();

    let mut current_state = "CLEAR".to_string();
    let mut child_detected = false;
    let mut current_temp = "--".to_string();
    let mut window_pos = "0".to_string();

    while let Ok(Some(line)) = reader.next_line().await {
        println!("[Guardian Output] {}", line);

        let mut level = "";
        if line.contains("CRITICAL") {
            level = "crit";
            current_state = "CRITICAL".to_string();
            child_detected = true;
            window_pos = "25".to_string();
        } else if line.contains("WARNING") {
            level = "warn";
            current_state = "WARNING".to_string();
            child_detected = true;
        } else if line.contains("MONITORING") {
            current_state = "MONITORING".to_string();
            child_detected = true;
        } else if line.contains("CLEAR") {
            current_state = "CLEAR".to_string();
            child_detected = false;
        }

        if line.contains("Window position") || line.contains("ACTUATION") || line.contains("mitigation") {
            level = "actuation";
        }

        // Extract temperature if found
        if let Some(pos) = line.find("°C") {
            let slice = &line[..pos];
            if let Some(num_start) = slice.rfind(|c: char| !c.is_numeric() && c != '.') {
                current_temp = slice[num_start + 1..].trim().to_string();
            }
        }

        let update = StateUpdate {
            r#type: "state_update".to_string(),
            state: current_state.clone(),
            child: child_detected,
            temp: current_temp.clone(),
            window: window_pos.clone(),
            log_text: Some(line),
            log_level: Some(level.to_string()),
        };

        if let Ok(json_str) = serde_json::to_string(&update) {
            let _ = tx.send(json_str);
        }
    }
}
