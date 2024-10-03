use tokio::net::TcpStream;
use tokio_tungstenite::client_async;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tokio::time::{sleep, Duration};
use crate::handlers::rx_command_handler::RxCommandHandler;
use crate::handlers::tx_command_handler::TxCommandHandler;
use crate::handlers::cli_handler::handle_cli;
use crate::shared_state::shared_state::SharedStateHandle;

pub async fn start_client(
    address: &str,
    passphrase: Arc<String>,
    no_exec: bool,
    no_transfer: bool,
    no_envelope: bool,
    shared_state: SharedStateHandle,
) {
    let shutdown_notify = Arc::new(Notify::new());

    loop {
        let shutdown_notify_clone = shutdown_notify.clone();
        let shared_state = Arc::clone(&shared_state);

        match connect_and_run(address, passphrase.clone(), no_exec, no_transfer, no_envelope, shutdown_notify_clone, shared_state).await {
            Ok(_) => eprintln!("Connection closed. Reconnecting in 5 seconds..."),
            Err(e) => eprintln!("Connection error: {}. Reconnecting in 5 seconds...", e),
        }
        sleep(Duration::from_secs(5)).await;
    }
}

async fn connect_and_run(
    address: &str,
    passphrase: Arc<String>,
    no_exec: bool,
    no_transfer: bool,
    no_envelope: bool,
    shutdown_notify: Arc<Notify>,
    shared_state: SharedStateHandle,
) -> Result<(), String> {
    let tcp_stream = TcpStream::connect(address)
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    let url = format!("ws://{}", address);
    let (ws_stream, _) = client_async(&url, tcp_stream)
        .await
        .map_err(|e| format!("WebSocket upgrade failed: {}", e))?;

    let (ws_sender, ws_receiver) = ws_stream.split();
    let ws_sender = Arc::new(Mutex::new(ws_sender));
    let ws_receiver = Arc::new(Mutex::new(ws_receiver));

    let tx_command_handler = Arc::new(Mutex::new(TxCommandHandler::new(
        passphrase.to_string(),
        Some(ws_sender.clone()),
        no_envelope,
        Arc::clone(&shared_state), 
    )));

    let rx_command_handler = Arc::new(Mutex::new(RxCommandHandler::new(
        passphrase.to_string(),
        Some(ws_sender.clone()),
        Some(ws_receiver.clone()),
        no_exec,
        no_transfer,
        no_envelope,
        Arc::clone(&shared_state),
    )));

    let rx_task = tokio::spawn({
        let rx_command_handler = rx_command_handler.clone();
        let shutdown_notify = shutdown_notify.clone();
        async move {
            let mut handler = rx_command_handler.lock().await;
            handler.handle_rx().await;
            shutdown_notify.notify_one();
        }
    });

    let cli_task = tokio::spawn({
        let tx_command_handler = tx_command_handler.clone();
        let shutdown_notify = shutdown_notify.clone();
        async move {
            handle_cli(tx_command_handler).await;
            shutdown_notify.notify_one();
        }
    });

    tokio::select! {
        _ = rx_task => Err("Message handling task ended.".to_string()),
        _ = cli_task => Err("CLI task ended.".to_string()),
        _ = shutdown_notify.notified() => {
            drop(rx_command_handler);
            drop(tx_command_handler);
            Err("Connection lost or tasks terminated.".to_string())
        }
    }
}
