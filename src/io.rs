use tokio::sync::Mutex;
use crate::rx_command_handler::RxCommandHandler;
use crate::tx_command_handler::TxCommandHandler;
use tokio_tungstenite::accept_async;
use futures_util::stream::StreamExt;
use crate::communication;
use tokio::io::{self, AsyncWriteExt, AsyncBufReadExt};
use tokio::net::TcpStream;
use std::sync::Arc;

pub async fn handle_cli(command_handler: Arc<Mutex<TxCommandHandler>>) {
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin).lines();

    print!("[*] UPLINK: CLI Handler is up and running. Enter commands below.\n\n");

    loop {
        match reader.next_line().await {
            Ok(Some(command)) => {
                let command = command.trim();
                if command.is_empty() {
                    continue;
                }

                let handler = command_handler.lock().await;
                handler.handle_command(command).await;
            }
            Ok(None) => {
                break;
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                break;
            }
        }
    }
}

pub async fn handle_client_connection(
    mut stream: TcpStream,
    passphrase: Arc<String>,
    no_exec: bool,
    no_transfer: bool,
) {
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
                    no_exec,
                    no_transfer,
                )));

                tokio::spawn(handle_cli(Arc::clone(&tx_command_handler)));

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
            }
        }
    } else {
        handle_http_request(stream).await;
    }
}

pub async fn handle_http_request(mut stream: TcpStream) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}",
        include_str!("static/index.html")
    );
    
    if let Err(e) = stream.write_all(response.as_bytes()).await {
        eprintln!("Failed to write HTTP response: {}", e);
    }

    if let Err(e) = stream.flush().await {
        eprintln!("Failed to flush HTTP response: {}", e);
    }
}
