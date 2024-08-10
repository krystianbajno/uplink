mod rx_command_handler;
mod tx_command_handler;

mod communication;
mod crypto;
mod compression;

mod file_operations;

use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, connect_async};
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use rx_command_handler::RxCommandHandler;
use tx_command_handler::TxCommandHandler;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = &args[1];
    let address = &args[2];

    let passphrase = Arc::new(std::env::var("PASSPHRASE").unwrap_or_else(|_| "default_passphrase".to_string()));

    match mode.as_str() {
        "server" => start_server(address, Arc::clone(&passphrase)).await,
        "client" => start_client(address, Arc::clone(&passphrase)).await,
        _ => eprintln!("Invalid mode. Use 'server' or 'client'"),
    }
}

async fn start_client(address: &str, passphrase: Arc<String>) {
    let (ws_stream, _) = connect_async(address)
        .await
        .expect("Failed to connect to server");

    let (ws_sender, ws_receiver) = ws_stream.split();

    let tx_command_handler = Arc::new(Mutex::new(TxCommandHandler::new(passphrase.clone().to_string(), Some(ws_sender.clone()), Some(ws_receiver.clone()))));
    let rx_command_handler = Arc::new(Mutex::new(RxCommandHandler::new(passphrase.clone().to_string(), Some(ws_sender), Some(ws_receiver))));

    let command_handler_for_cli = Arc::clone(&tx_command_handler);
    tokio::spawn(communication::handle_cli(Arc::clone(&command_handler_for_cli)));

    let mut command_handler_for_ws = rx_command_handler.lock().await;
    tokio::spawn(async move {
        command_handler_for_ws.handle_responses().await;
    });
}

async fn start_server(bind_addr: &str, passphrase: Arc<String>) {
    let listener = TcpListener::bind(bind_addr).await.unwrap();
    println!("Server listening on {}", bind_addr);

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream)
            .await
            .expect("Error during WebSocket handshake");

        let (ws_sender, ws_receiver) = ws_stream.split();

        let tx_command_handler = Arc::new(Mutex::new(TxCommandHandler::new(passphrase.clone().to_string(), Some(ws_sender.clone()), Some(ws_receiver.clone()))));
        let rx_command_handler = Arc::new(Mutex::new(RxCommandHandler::new(passphrase.clone().to_string(), Some(ws_sender.clone()), Some(ws_receiver.clone()))));

        let command_handler_for_cli = Arc::clone(&tx_command_handler);
        tokio::spawn(communication::handle_cli(Arc::clone(&command_handler_for_cli)));

        let mut command_handler_for_ws = rx_command_handler.lock().await;
        tokio::spawn(async move {
            command_handler_for_ws.handle_responses().await;
        });
    }
}
