mod rx_command_handler;
mod tx_command_handler;

mod response_handler;
mod command;
mod communication;
mod crypto;
mod compression;

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, client_async};
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use rx_command_handler::RxCommandHandler;
use tx_command_handler::TxCommandHandler;

#[tokio::main]
async fn main() {
    let (mode, address, passphrase) = get_config();
    let passphrase = Arc::new(passphrase);

    match mode.as_deref() {
        Some("server") => {
            let address = address.expect("Address is required for server mode");
            start_server(&address, Arc::clone(&passphrase)).await;
        }
        Some("client") => {
            let address = address.expect("Address is required for client mode");
            start_client(&address, Arc::clone(&passphrase)).await;
        }
        _ => eprintln!("Invalid or missing mode. Use 'server' or 'client'"),
    }
}

fn get_config() -> (Option<String>, Option<String>, String) {
    let precompiled_mode: Option<&str> = option_env!("CARGO_PKG_METADATA_PRECOMPILED_MODE");
    let precompiled_address: Option<&str> = option_env!("CARGO_PKG_METADATA_PRECOMPILED_ADDRESS");
    let precompiled_passphrase: Option<&str> = option_env!("CARGO_PKG_METADATA_PRECOMPILED_PASSPHRASE");

    let args: Vec<String> = std::env::args().collect();

    let mode = args.get(1).cloned().or_else(|| {
        precompiled_mode.map(|s| s.trim_matches('"').to_string())
    });

    let address = args.get(2).cloned().or_else(|| {
        precompiled_address.map(|s| s.trim_matches('"').to_string())
    });

    let passphrase = std::env::var("PASSPHRASE")
        .or_else(|_| {
            precompiled_passphrase
                .map(|s| s.trim_matches('"').to_string())
                .ok_or(std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| "default_passphrase".to_string());

    (mode, address, passphrase)
}

async fn start_client(address: &str, passphrase: Arc<String>) {
    let tcp_stream = TcpStream::connect(address).await.expect("Failed to connect to server");
    let url = format!("ws://{}", address);
    let (ws_stream, _) = client_async(&url, tcp_stream)
        .await
        .expect("Failed to upgrade to WebSocket");

    let (ws_sender, ws_receiver) = ws_stream.split();
    let ws_sender = Arc::new(Mutex::new(ws_sender));
    let ws_receiver = Arc::new(Mutex::new(ws_receiver));

    let tx_command_handler = Arc::new(Mutex::new(TxCommandHandler::new(
        passphrase.clone().to_string(),
        Some(Arc::clone(&ws_sender)),
        Some(Arc::clone(&ws_receiver)),
    )));
    let rx_command_handler = Arc::new(Mutex::new(RxCommandHandler::new(
        passphrase.clone().to_string(),
        Some(Arc::clone(&ws_sender)),
        Some(Arc::clone(&ws_receiver)),
    )));

    tokio::spawn(communication::handle_cli(Arc::clone(&tx_command_handler)));

    tokio::spawn(async move {
        let mut command_handler_for_ws = rx_command_handler.lock().await;
        command_handler_for_ws.handle_rx().await;
    });

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

async fn start_server(bind_addr: &str, passphrase: Arc<String>) {
    let listener = TcpListener::bind(bind_addr).await.unwrap();
    println!("Server listening on {}", bind_addr);

    while let Ok((mut stream, _)) = listener.accept().await {
        if communication::is_websocket_upgrade_request(&mut stream).await {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    let (ws_sender, ws_receiver) = ws_stream.split();
                    let ws_sender = Arc::new(Mutex::new(ws_sender));
                    let ws_receiver = Arc::new(Mutex::new(ws_receiver));

                    let tx_command_handler = Arc::new(Mutex::new(TxCommandHandler::new(
                        passphrase.clone().to_string(),
                        Some(Arc::clone(&ws_sender)),
                        Some(Arc::clone(&ws_receiver)),
                    )));
                    let rx_command_handler = Arc::new(Mutex::new(RxCommandHandler::new(
                        passphrase.clone().to_string(),
                        Some(Arc::clone(&ws_sender)),
                        Some(Arc::clone(&ws_receiver)),
                    )));

                    tokio::spawn(communication::handle_cli(Arc::clone(&tx_command_handler)));

                    tokio::spawn(async move {
                        let mut command_handler_for_ws = rx_command_handler.lock().await;
                        command_handler_for_ws.handle_rx().await;
                    });

                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    }
                }
                Err(e) => {
                    eprintln!("WebSocket handshake failed: {:?}", e);
                    continue;
                }
            }
        } else {
            // Handle as an HTTP request
            communication::handle_http_request(stream).await;
        }
    }
}
