mod rx_command_handler;
mod tx_command_handler;

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

    match mode.as_str() {
        "server" => start_server(&address, Arc::clone(&passphrase)).await,
        "client" => start_client(&address, Arc::clone(&passphrase)).await,
        _ => eprintln!("Invalid mode. Use 'server' or 'client'"),
    }
}

fn get_config() -> (String, String, String) {
    #[cfg(feature = "precompiled_mode")]
    let mode = env!("CARGO_PKG_METADATA_PRECOMPILED_MODE").to_string();

    #[cfg(feature = "precompiled_address")]
    let address = env!("CARGO_PKG_METADATA_PRECOMPILED_ADDRESS").to_string();

    #[cfg(feature = "precompiled_passphrase")]
    let passphrase = env!("CARGO_PKG_METADATA_PRECOMPILED_PASSPHRASE").to_string();

    #[cfg(not(feature = "precompiled_mode"))]
    let mode = {
        let args: Vec<String> = std::env::args().collect();
        if args.len() > 1 {
            args[1].clone()
        } else {
            eprintln!("Usage: <mode> <address>");
            std::process::exit(1);
        }
    };

    #[cfg(not(feature = "precompiled_address"))]
    let address = {
        let args: Vec<String> = std::env::args().collect();
        if args.len() > 2 {
            args[2].clone()
        } else {
            eprintln!("Usage: <mode> <address>");
            std::process::exit(1);
        }
    };

    #[cfg(not(feature = "precompiled_passphrase"))]
    let passphrase = {
        std::env::var("PASSPHRASE").unwrap_or_else(|_| "default_passphrase".to_string())
    };

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
}

async fn start_server(bind_addr: &str, passphrase: Arc<String>) {
    let listener = TcpListener::bind(bind_addr).await.unwrap();
    println!("Server listening on {}", bind_addr);

    while let Ok((stream, _)) = listener.accept().await {
        if communication::is_http_request(&stream).await {
            communication::handle_http_request(stream).await;
            continue;
        }

        let ws_stream = accept_async(stream)
            .await
            .expect("Error during WebSocket handshake");

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
    }
}